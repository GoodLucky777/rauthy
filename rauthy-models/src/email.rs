use crate::app_state::AppState;
use crate::entity::magic_links::MagicLinkPassword;
use crate::entity::users::User;
use actix_web::web;
use askama_actix::Template;
use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication;
use lettre::{AsyncSmtpTransport, AsyncTransport};
use rauthy_common::constants::{SMTP_FROM, SMTP_PASSWORD, SMTP_URL, SMTP_USERNAME};
use std::time::Duration;
use time::OffsetDateTime;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error, info, warn};

#[derive(Debug)]
pub struct EMail {
    pub address: String,
    pub subject: String,
    pub text: String,
    pub html: Option<String>,
}

#[derive(Default, Template)]
#[template(path = "email/reset.html")]
pub struct EMailResetHtml<'a> {
    pub pub_url: &'a str,
    pub link: &'a str,
    pub exp: &'a str,
}

#[derive(Default, Template)]
#[template(path = "email/reset.txt")]
pub struct EmailResetTxt<'a> {
    pub pub_url: &'a str,
    pub link: &'a str,
    pub exp: &'a str,
}

#[derive(Default, Template)]
#[template(path = "email/reset_info.html")]
pub struct EMailResetInfoHtml<'a> {
    pub pub_url: &'a str,
    pub link: &'a str,
    pub exp: &'a str,
}

#[derive(Default, Template)]
#[template(path = "email/reset_info.txt")]
pub struct EmailResetInfoTxt<'a> {
    pub pub_url: &'a str,
    pub link: &'a str,
    pub exp: &'a str,
}

pub async fn send_pwd_reset(
    data: &web::Data<AppState>,
    magic_link: &MagicLinkPassword,
    user: &User,
) {
    let link = format!(
        "{}/users/{}/reset/{}",
        data.issuer, magic_link.user_id, &magic_link.id
    );
    let exp = OffsetDateTime::from_unix_timestamp(magic_link.exp)
        .unwrap()
        .to_string();

    let text = EmailResetTxt {
        pub_url: &data.public_url,
        link: &link,
        exp: &exp,
    };

    let html = EMailResetHtml {
        pub_url: &data.public_url,
        link: &link,
        exp: &exp,
    };

    let req = EMail {
        address: user.email.to_string(),
        subject: format!(
            "Password Reset Request - {} {}",
            user.given_name, user.family_name
        ),
        text: text.render().expect("Template rendering: EmailResetTxt"),
        html: Some(html.render().expect("Template rendering: EmailResetHtml")),
    };

    let tx = &data.tx_email;
    let res = tx.send_timeout(req, Duration::from_secs(10)).await;
    match res {
        Ok(_) => {}
        Err(ref e) => {
            error!(
                "Error sending magic link email request for user '{}': {:?}",
                user.email, e
            );
        }
    }
    if res.is_err() {}
}

pub async fn send_pwd_reset_info(data: &web::Data<AppState>, user: &User) {
    let exp = OffsetDateTime::from_unix_timestamp(user.password_expires.unwrap())
        .expect("Corrupt user password expiry timestamp");
    let link = format!("{}/auth/v1/account.html", data.public_url);

    let text = EmailResetInfoTxt {
        pub_url: &data.public_url,
        link: &link,
        exp: &exp.to_string(),
    };

    let html = EMailResetInfoHtml {
        pub_url: &data.public_url,
        link: &link,
        exp: &exp.to_string(),
    };

    let req = EMail {
        address: user.email.to_string(),
        subject: "Password is about to expire".to_string(),
        text: text
            .render()
            .expect("Template rendering: EmailResetInfoTxt"),
        html: Some(
            html.render()
                .expect("Template rendering: EmailResetInfoHtml"),
        ),
    };

    let tx = &data.tx_email;
    let res = tx.send_timeout(req, Duration::from_secs(10)).await;
    match res {
        Ok(_) => {}
        Err(ref e) => {
            error!(
                "Error sending magic link email request for user '{}': {:?}",
                user.email, e
            );
        }
    }
    if res.is_err() {}
}

pub async fn sender(mut rx: Receiver<EMail>, test_mode: bool) {
    debug!("E-Mail sender started");

    // to make the integration tests not panic, results are taken and just thrown away
    // not the nicest approach for now, but it works
    if test_mode {
        loop {
            let req = rx.recv().await;
            if req.is_some() {
                debug!(
                    "New E-Mail for address: {:?}",
                    req.as_ref().unwrap().address
                );
            } else {
                warn!("Received 'None' in email 'sender' - exiting");
                return;
            }
        }
    }

    let creds = authentication::Credentials::new(SMTP_USERNAME.clone(), SMTP_PASSWORD.clone());
    let mailer = AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&SMTP_URL)
        .expect("Connection Error with 'SMTP_URL'")
        .credentials(creds)
        .build();

    loop {
        debug!("Listening for incoming send E-Mail requests");
        if let Some(req) = rx.recv().await {
            debug!("New E-Mail for address: {:?}", req.address);

            let to = format!("{} <{}>", req.subject, req.address);

            let email = if let Some(html) = req.html {
                lettre::Message::builder()
                    .from(
                        SMTP_FROM
                            .parse()
                            .expect("SMTP_FROM could not be parsed correctly"),
                    )
                    .to(to.parse().unwrap())
                    .subject(req.subject)
                    .multipart(MultiPart::alternative_plain_html(req.text, html))
            } else {
                lettre::Message::builder()
                    .from(
                        SMTP_FROM
                            .parse()
                            .expect("SMTP_FROM could not be parsed correctly"),
                    )
                    .to(to.parse().unwrap())
                    .subject(req.subject)
                    .singlepart(SinglePart::plain(req.text))
            };

            if email.is_err() {
                error!("Error building the E-Mail to '{}'", req.address);
            } else {
                match mailer.send(email.unwrap()).await {
                    Ok(_) => info!("E-Mail to '{}' sent successfully!", req.address),
                    Err(e) => error!("Could not send E-Mail: {:?}", e),
                }
            }
        } else {
            warn!("Received 'None' in email 'sender' - exiting");
            break;
        }
    }
}
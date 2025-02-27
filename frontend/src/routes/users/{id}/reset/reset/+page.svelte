<script>
    import {run} from 'svelte/legacy';

    import * as yup from "yup";
    import {onMount} from "svelte";
    import {
        arrBufToBase64UrlSafe,
        base64UrlSafeToArrBuf,
        extractFormErrors,
        generatePassword,
        getQueryParams
    } from "../../../../../utils/helpers.js";
    import {
        resetPassword,
        webauthnAuthStart,
        webauthnRegStartAccReset,
        webauthnRegFinishAccReset,
    } from "../../../../../utils/dataFetching.js";
    import Loading from "$lib/Loading.svelte";
    import Button from "$lib/Button.svelte";
    import PasswordPolicy from "../../../../../components/passwordReset/PasswordPolicy.svelte";
    import Input from "$lib/inputs/Input.svelte";
    import PasswordInput from "$lib/inputs/PasswordInput.svelte";
    import WebauthnRequest from "../../../../../components/webauthn/WebauthnRequest.svelte";
    import BrowserCheck from "../../../../../components/BrowserCheck.svelte";
    import WithI18n from "$lib/WithI18n.svelte";
    import {slide} from "svelte/transition";
    import LangSelector from "$lib/LangSelector.svelte";
    import {REGEX_NAME} from "../../../../../utils/constants.js";

    const btnWidth = '150px';
    const inputWidth = '320px';

    let t = $state();
    let csrf = '';
    let policy = $state();
    let isReady = $state(true);
    // let isReady = false;
    let isMfa = false;

    let isLoading = $state(false);
    let err = $state('');
    let userId = '';
    // let requestType = '';
    let requestType = $state("password_reset")
    let accountTypeNew = $state('');
    let magicLinkId = '';
    let redirectUri = $state();
    let success = $state(false);
    let accepted = $state(false);
    let showCopy = $state(false);
    let webauthnData = $state();

    let formValues = $state({
        passkeyName: '',
        password: '',
        passwordConfirm: '',
    });
    let formErrors = $state({});

    let schemaPasskey = $state();
    let schemaPassword = $state();


    onMount(async () => {
        // const policy_vals = '10, 128, 1, 1, 1, 1, 3';
        const policy_vals = document.getElementsByName('rauthy-data')[0].id;
        const arr = [];
        policy_vals
            .split(',')
            .forEach(v => arr.push(v));
        policy = {
            length_min: Number.parseInt(arr[0]),
            length_max: Number.parseInt(arr[1]),
            include_lower_case: Number.parseInt(arr[2]),
            include_upper_case: Number.parseInt(arr[3]),
            include_digits: Number.parseInt(arr[4]),
            include_special: Number.parseInt(arr[5]),
            not_recently_used: Number.parseInt(arr[6]),
        };
        isMfa = arr[7] == "true";

        csrf = window.document.getElementsByName('rauthy-csrf-token')[0].id
        userId = window.location.href.split("/users/")[1].split("/")[0];
        magicLinkId = window.location.href.split("/reset/")[1].split("?")[0];

        const params = getQueryParams();
        requestType = params['type'];

        isReady = true;
    })

    function navigateToAccount() {
        window.location.replace('/auth/v1/account');
    }

    function generate() {
        const len = policy.length_min > 24 ? policy.length_min : 24;
        let pwd = generatePassword(
            len, policy.include_lower_case,
            policy.include_upper_case,
            policy.include_digits,
            policy.include_special,
        );
        formValues.password = pwd;
        formValues.passwordConfirm = pwd;
    }

    async function handleRegisterPasskey() {
        err = '';

        try {
            await schemaPasskey.validate(formValues, {abortEarly: false});
            formErrors = {};
        } catch (err) {
            formErrors = extractFormErrors(err);
            return;
        }

        const passkeyName = formValues.passkeyName;
        if (passkeyName.length < 2) {
            err = t.mfa.passkeyNameErr;
            return;
        }

        let data = {
            passkey_name: passkeyName,
            magic_link_id: magicLinkId,
        };
        let res = await webauthnRegStartAccReset(userId, data, csrf);
        if (res.status === 200) {
            let challenge = await res.json();

            // we need to force UV at this point in the browser already to have a better UV
            challenge.publicKey.authenticatorSelection.userVerification = 'required';

            // the navigator credentials engine needs some values as array buffers
            challenge.publicKey.challenge = base64UrlSafeToArrBuf(challenge.publicKey.challenge);
            challenge.publicKey.user.id = base64UrlSafeToArrBuf(challenge.publicKey.user.id);
            challenge.publicKey.excludeCredentials = challenge.publicKey.excludeCredentials

            if (challenge.publicKey.excludeCredentials) {
                challenge.publicKey.excludeCredentials = challenge.publicKey.excludeCredentials.map(cred => {
                    cred.id = base64UrlSafeToArrBuf(cred.id);
                    return cred;
                });
            }

            // prompt for the user security key and get its public key
            let challengePk = await navigator.credentials.create(challenge);

            // the backend expects base64 url safe string instead of array buffers
            let data = {
                passkey_name: passkeyName,
                data: {
                    id: challengePk.id,
                    rawId: arrBufToBase64UrlSafe(challengePk.rawId),
                    response: {
                        attestationObject: arrBufToBase64UrlSafe(challengePk.response.attestationObject),
                        clientDataJSON: arrBufToBase64UrlSafe(challengePk.response.clientDataJSON),
                    },
                    type: challengePk.type,
                },
                magic_link_id: magicLinkId,
            }

            // send the keys' pk to the backend and finish the registration
            res = await webauthnRegFinishAccReset(userId, data, csrf);
            if (res.status === 201) {
                formValues = {
                    passkeyName: '',
                    password: '',
                    passwordConfirm: '',
                };
                success = true;
            } else {
                onWebauthnError();
                console.error(res);
            }
        } else {
            onWebauthnError();
            let body = await res.json();
            console.error(body.error);
            console.error(body.message);
        }
    }

    async function passwordReset() {
        try {
            await schemaPassword.validate(formValues, {abortEarly: false});
            formErrors = {};
        } catch (err) {
            formErrors = extractFormErrors(err);
            return;
        }

        if (!accepted) {
            return;
        }

        if (formValues.password.length > 256) {
            err = 'max 256';
            return;
        }

        // do passwords match?
        if (formValues.password !== formValues.passwordConfirm) {
            err = t.passwordNoMatch;
            return;
        } else {
            err = '';
        }

        if (isMfa) {
            let res = await webauthnAuthStart(userId, {purpose: 'PasswordReset'});
            let body = await res.json();
            if (!res.ok) {
                err = body.message;
                isLoading = false;
                return;
            }

            if (body.user_id !== userId) {
                err = 'MFA user ID does not match - this should never happen';
                isLoading = false;
                return;
            }

            webauthnData = body;
        } else {
            await onSubmitFinish();
        }
    }

    async function onSubmitFinish(mfaCode) {
        isLoading = true;

        const data = {
            password: formValues.password,
            magic_link_id: magicLinkId,
            mfa_code: mfaCode,
        };

        const res = await resetPassword(userId, data, csrf);
        if (res.ok) {
            err = '';
            formValues = {
                passkeyName: '',
                password: '',
                passwordConfirm: '',
            };
            redirectUri = res.headers.get('Location');
            console.log('redirectUri: ' + redirectUri);
            success = true;
        } else {
            const body = await res.json();
            err = body.message;
        }

        isLoading = false;
    }

    function onWebauthnError() {
        // If there is any error with the key, the user should start a new login process
        webauthnData = undefined;
        err = t.mfa.errorReg;
    }

    function onWebauthnSuccess(res) {
        if (res) {
            webauthnData = undefined;
            onSubmitFinish(res.code)
        }
    }

    run(() => {
        if (t) {
            schemaPasskey = yup.object().shape({
                passkeyName: yup.string()
                    .required(t.required)
                    .matches(REGEX_NAME, t.mfa.passkeyNameErr),
            });
            schemaPassword = yup.object().shape({
                password: yup.string().required(t.required),
                passwordConfirm: yup.string().required(t.required)
            });
        }
    });
    run(() => {
        if (accountTypeNew) {
            // reset all possibly filled in form values from before
            formValues = {
                passkeyName: '',
                password: '',
                passwordConfirm: '',
            };
        }
    });
    run(() => {
        if (formValues.password?.length > 0 && formValues.password === formValues.passwordConfirm) {
            showCopy = true;
        }
    });
    run(() => {
        if (success) {
            setTimeout(() => {
                if (redirectUri) {
                    window.location.replace(redirectUri);
                } else {
                    navigateToAccount();
                }
            }, 5000);
        }
    });
</script>

<svelte:head>
    <!-- the :head component cannot be wrapped inside the <WithI18n> unfortunately -->
    {#if t}
        {#if requestType.startsWith('new_user')}
            <title>{t.newAccount}</title>
        {:else if requestType === "password_reset"}
            <title>{t.passwordReset}</title>
        {/if}
    {:else}
        <title>Password</title>
    {/if}

</svelte:head>

<BrowserCheck>
    {#if !isReady}
        <Loading/>
    {/if}

    <WithI18n bind:t content="passwordReset">
        <div class="container">
            {#if requestType.startsWith('new_user')}
                {#if webauthnData}
                    <WebauthnRequest
                            bind:data={webauthnData}
                            onSuccess={onWebauthnSuccess}
                            onError={onWebauthnError}
                    />
                {/if}

                <h1>{t.newAccount}</h1>
                <p>{t.newAccDesc1}</p>
                <p>{t.newAccDesc2}<a href={t.fidoLink} target="_blank">FIDO Alliance</a></p>

                <div style:margin-bottom="1rem">
                    <Button
                            on:click={() => accountTypeNew = "passkey"}
                            width={btnWidth}
                            bind:isLoading
                            level={2}
                            isDisabled={success}
                    >
                        {t.passwordless.toUpperCase()}
                    </Button>
                    <Button
                            on:click={() => accountTypeNew = "password"}
                            width={btnWidth}
                            bind:isLoading
                            level={3}
                            isDisabled={success}
                    >
                        {t.password.toUpperCase()}
                    </Button>
                </div>

                {#if accountTypeNew === "password"}
                    <div transition:slide>
                        <PasswordPolicy {t} bind:accepted {policy} bind:password={formValues.password}/>

                        <PasswordInput
                                bind:value={formValues.password}
                                error={formErrors.password}
                                autocomplete="new-password"
                                placeholder={t.password}
                                width={inputWidth}
                                {showCopy}
                                disabled={success}
                        >
                            {t.password.toUpperCase()}
                        </PasswordInput>
                        <PasswordInput
                                bind:value={formValues.passwordConfirm}
                                error={formErrors.passwordConfirm}
                                autocomplete="new-password"
                                placeholder={t.passwordConfirm}
                                width={inputWidth}
                                {showCopy}
                                disabled={success}
                        >
                            {t.passwordConfirm.toUpperCase()}
                        </PasswordInput>

                        <Button
                                on:click={generate}
                                width={btnWidth}
                                level={3}
                                isDisabled={success}
                        >
                            {t.generate.toUpperCase()}
                        </Button>
                        <Button
                                on:click={passwordReset}
                                width={btnWidth}
                                bind:isLoading level={2}
                                isDisabled={success}
                        >
                            {t.save.toUpperCase()}
                        </Button>

                        {#if success}
                            <div class="success">
                                {t.success1}
                                <br>
                                {t.success2}
                            </div>
                        {/if}
                    </div>
                {:else if accountTypeNew === "passkey"}
                    <div transition:slide>
                        <Input
                                bind:value={formValues.passkeyName}
                                bind:error={formErrors.passkeyName}
                                autocomplete="off"
                                placeholder={t.mfa.passkeyName}
                                on:enter={handleRegisterPasskey}
                                width={inputWidth}
                                disabled={success}
                        >
                            {t.mfa.passkeyName}
                        </Input>
                        <Button
                                on:click={handleRegisterPasskey} width={btnWidth}
                                level={success ? 2 : 1}
                                isDisabled={success}
                        >
                            {t.mfa.register.toUpperCase()}
                        </Button>

                        {#if success}
                            <div class="success">
                                <p>{t.successPasskey1}</p>
                                <p>{t.successPasskey2}</p>
                                <Button on:click={navigateToAccount} width={btnWidth} level={1}>
                                    {t.accountLogin.toUpperCase()}
                                </Button>
                            </div>
                        {/if}
                    </div>
                {/if}
            {:else if requestType.startsWith('password_reset')}
                {#if webauthnData}
                    <WebauthnRequest
                            bind:data={webauthnData}
                            purpose="PasswordReset"
                            onSuccess={onWebauthnSuccess}
                            onError={onWebauthnError}
                    />
                {/if}

                <h1>Password Reset</h1>

                <PasswordPolicy {t} bind:accepted {policy} bind:password={formValues.password}/>

                <PasswordInput
                        bind:value={formValues.password}
                        error={formErrors.password}
                        autocomplete="new-password"
                        placeholder={t.password}
                        width={inputWidth}
                        {showCopy}
                >
                    {t.password.toUpperCase()}
                </PasswordInput>
                <PasswordInput
                        bind:value={formValues.passwordConfirm}
                        error={formErrors.passwordConfirm}
                        autocomplete="new-password"
                        placeholder={t.passwordConfirm}
                        width={inputWidth}
                        {showCopy}
                >
                    {t.passwordConfirm.toUpperCase()}
                </PasswordInput>

                <Button on:click={generate} width={btnWidth} level={3}>
                    {t.generate.toUpperCase()}
                </Button>
                <Button on:click={passwordReset} width={btnWidth} bind:isLoading level={2}>
                    {t.save.toUpperCase()}
                </Button>

                {#if success}
                    <div class="success">
                        {t.success1}
                        {t.success2}
                        <br>
                        {t.success3}
                        <br>
                        <a href={redirectUri || '/auth/v1/account'}>Link</a>
                    </div>
                {/if}
            {/if}

            {#if err}
                <div class="err">
                    {err}
                </div>
            {/if}
        </div>

        <LangSelector absolute/>
    </WithI18n>
</BrowserCheck>

<style>
    a {
        color: var(--col-act2);
    }

    a:visited {
        color: var(--col-act2);
    }

    .container {
        display: flex;
        flex-direction: column;
        justify-content: center;
        max-width: 20.5rem;
    }

    .err {
        margin: 0 5px;
        color: var(--col-err);
    }

    .success {
        margin: 5px;
        color: var(--col-acnt);
    }
</style>

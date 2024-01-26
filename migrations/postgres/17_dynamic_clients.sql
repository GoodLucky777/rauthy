create table clients_dyn
(
    id                 varchar not null
        constraint clients_dyn_pk
            primary key
        constraint clients_dyn_clients_id_fk
            references clients
            on update cascade on delete cascade,
    created            integer not null,
    last_used          integer,
    registration_token varchar not null
);

create index clients_dyn_last_used_index
    on clients_dyn (last_used);

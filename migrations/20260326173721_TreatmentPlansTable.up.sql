-- Add up migration script here
create table treatment_plans(
    tp_id serial4 primary key,
    client int4 references clients (cl_id),
    author int4 references users (id),
    date_created date not null,
    client_name text not null,
    author_name text not null,
    is_active bool not null default true
)

-- Add up migration script here
create table treatment_plans(
    tp_id serial4 primary key,
    client int4 references clients (cl_id),
    author int4 references consultants (co_id),
    date_created date not null
)

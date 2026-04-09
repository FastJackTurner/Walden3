-- Add up migration script here
create table schedule(
    appt_id serial4 primary key,
    co_id int4 references consultants (co_id),
    cl_id int4 references clients (cl_id),
    te_id int4 references technicians (te_id),
    appt_date date not null,
    appt_time time not null
)

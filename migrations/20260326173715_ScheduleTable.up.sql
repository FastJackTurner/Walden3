-- Add up migration script here
create table if not exists schedule (
    appt_id serial4 primary key,
    client_id int4 references clients (cl_id),
    therapist_id int4 references users (id),
    appt_date date not null,
    appt_time time not null,
    appt_type appt_type not null default 'direct_service'
)

-- Add up migration script here
create table TechSchedule(
    appt_id serial4 primary key,
    client_id int4 references clients (cl_id),
    technician_id int4 references users (id),
    appt_date date not null,
    appt_time time not null
)

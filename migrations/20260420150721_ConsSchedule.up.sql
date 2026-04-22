-- Add up migration script here
create table ConsSchedule(
    appt_id serial4 primary key,
    cons_id int4 references users (id),
    cl_id int4 references clients (cl_id),
    appt_date date not null,
    appt_time time not null,
    parent_training bool not null default false
)

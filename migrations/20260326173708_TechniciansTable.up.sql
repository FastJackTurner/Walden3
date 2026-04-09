-- Add up migration script here
create table technicians(
    te_id serial4 primary key,
    co_id int4 references consultants (co_id),
    c_f_name text not null,
    c_l_name text not null
)

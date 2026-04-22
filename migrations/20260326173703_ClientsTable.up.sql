-- Add up migration script here
create table clients(
    cl_id serial4 primary key,
    co_id int4 references users (id),
    cl_f_name text not null,
    cl_l_name text not null
)

-- Add up migration script here
create table consultants (
    co_id serial4 primary key,
    co_f_name text not null,
    co_l_name text not null
)

-- Add up migration script here
create table users (
    id serial4 primary key,
    username text,
    password_hash text,
    role user_role not null default 'consultant',
    f_name text not null default 'Jack',
    l_name text not null default 'Turner',
    consultant_id int4,
    constraint must_be_tech
        check (
            (role = 'tech' and consultant_id IS NOT NULL) OR
            (role = 'consultant' and consultant_id IS NULL) OR
            (role = 'admin' and consultant_id IS NULL)
        )
)

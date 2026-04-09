-- Add up migration script here
create table goals(
    go_id serial4 primary key,
    tp_id int4 references treatment_plans (tp_id),
    is_active bool not null default true,
    data_type text not null,
    goal_name text not null,
    teaching_procedures text not null,
    created_at date not null
)

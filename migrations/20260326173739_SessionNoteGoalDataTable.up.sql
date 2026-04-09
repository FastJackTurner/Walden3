-- Add up migration script here
create table session_notes_goal_data(
    go_id serial4 primary key,
    tp_id int4 references treatment_plans (tp_id),
    session_date date not null,
    data_numerator int4 not null,
    data_denominator int4 default 1 not null,
    comment text not null
)

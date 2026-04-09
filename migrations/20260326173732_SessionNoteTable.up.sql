-- Add up migration script here
create table session_notes(
    tp_id int4 references treatment_plans (tp_id),
    comment text not null,
    techniqes_used text not null
)

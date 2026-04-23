-- Add up migration script here
create table session_notes(
    note_id serial4,
    tp_id int4 references treatment_plans (tp_id),
    appt_id int4 references schedule (appt_id),
    comment text not null,
    techniqes_used text not null
)

use askama::Template;
use axum::extract::State;
use axum::Form;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::{Deserialize};
use std::sync::Arc;
use recursive::recursive;
use chrono;

use crate::db::Consultant;
use crate::app::AppState;


#[derive(Template, sqlx::FromRow)]
#[template(path = "error_template.html")]
struct ErrorTemplate {
    error: String
}


//-------------------------------------------------------------
// SQLTEST:

pub async fn sql_test(State(pool): State<Arc<AppState>>) -> String {

    let oggy: Result<Consultant, sqlx::Error> =
    sqlx::query_as(
    r#"
        Select * from consultants where co_id = 0;
    "#)
    .fetch_one(&pool.pool)
   .await;

    match oggy {
        Ok(oggy) => oggy.co_f_name,
        Err(e) => e.to_string()
    }
}

//------------------------------------------------------------
// CREATEANDPOSTGOALS:
//
#[derive(Deserialize, sqlx::FromRow, Debug)]
pub struct PartialGoal {
    tp_id: i32,
    data_type: String,
    goal_name: String,
    teaching_procedures: String
}

#[derive(Deserialize, sqlx::FromRow, Debug, serde::Serialize)]
pub struct Goal {
     pub go_id: i32,
     pub tp_id: i32,
     pub is_active: bool,
     pub data_type: String,
     pub goal_name: String,
     pub teaching_procedures: String,
     pub created_at: chrono::NaiveDate,
     
}

pub async fn post_goal(State(pool): State<Arc<AppState>>, Form(goal): Form<PartialGoal>) -> String {
    let full_goal = Goal {
        go_id: 0,
        tp_id: goal.tp_id,
        is_active: true,
        data_type: goal.data_type,
        goal_name : goal.goal_name,
        teaching_procedures: goal.teaching_procedures,
        created_at: chrono::Utc::now().date_naive(),
    };
    let goal_insert = sqlx::query(
        r#"
           insert into goals(tp_id, is_active, data_type, goal_name, teaching_procedures, created_at) values
           ($1, $2, $3, $4, $5, $6)
           returning go_id
        "#)
        .bind(full_goal.tp_id)
        .bind(full_goal.is_active)
        .bind(full_goal.data_type)
        .bind(full_goal.goal_name)
        .bind(full_goal.teaching_procedures)
        .bind(full_goal.created_at)
        .execute(&pool.pool)
        .await;

    match goal_insert {
        Ok(_) => return String::from("Success"),
        Err(e) => return e.to_string()
    }
}

#[derive(Template)]
#[template(path = "goal-list.html")]
pub struct Goals{
    pub goals: Vec<String>,
}


#[recursive]
fn create_goals(num: &mut u32) -> Vec<String> {
    let mut ret_vec = vec![];
    while *num != 0{
        ret_vec.push(String::from("foo"));
        *num -=1;
    }
    // if *num == 0 {
        
    // } else {
    //     ret_vec.push(String::from("foo"));
    //     println!("goal");
    //     create_goals(&mut (*num - 1));
    // }
    
    // println!("goal_vec len: {}", &ret_vec.len());
    ret_vec
}


#[derive(Deserialize)]
pub struct GoalNumbers {
    pub goal_numbers: u32
}
pub async fn create_goal(Form(mut num): Form<GoalNumbers>) -> impl IntoResponse {
    let goals_vec = create_goals(&mut num.goal_numbers);
    
    let template = Goals{
        goals: goals_vec,
    };
    HtmlTemplate(template)
}

//-----------------------------------------------------------
// VIEWGOALS:

#[derive(Template, sqlx::FromRow, Deserialize, serde::Serialize)]
#[template(path = "view_goals.html")]
pub struct GoalsTemplate {
    pub goals: Vec<Goal>
}


#[derive(Deserialize)]
pub struct TreatmentPlanId {
    pub tp_id: u32
}

#[axum::debug_handler]
pub async fn view_goals(State(pool): State<Arc<AppState>>,
    Form(num): Form<TreatmentPlanId>,) -> impl IntoResponse {
    let num = num.tp_id as i32;
    let rows:Result< Vec<_>, sqlx::Error> = sqlx::query_as(
    "
        select * from goals where tp_id = $1
    ")
    .bind(num)
    .fetch_all(&pool.pool)
    .await;


    match rows {
        Ok(goals) => {
            if goals.len() < 1 {
            let goal = Goal{
                go_id: 420,
                tp_id: 69,
                is_active: false,
                data_type: String::from("error is:"),
                goal_name: "You typed in the wrong treatment plan id".to_string(),
                teaching_procedures: String::new(),
                created_at: chrono::NaiveDate::from_ymd_opt(2024, 12, 12).unwrap(),
            };
            let template = GoalsTemplate {
                goals: vec![goal]
            }; HtmlTemplate(template)
            }else {
            let template = GoalsTemplate {
            goals
          }; HtmlTemplate(template)}
        },
        Err(e) => {
            let goal = Goal{
                go_id: 420,
                tp_id: 69,
                is_active: false,
                data_type: String::from("error is:"),
                goal_name: e.to_string(),
                teaching_procedures: String::new(),
                created_at: chrono::NaiveDate::from_ymd_opt(2024, 12, 12).unwrap(),
            };
            let template = GoalsTemplate {
                goals: vec![goal]
            }; HtmlTemplate(template)
        }, 
    }

}


//------------------------------------------------------------
// MAKESCHEDULE:

#[derive(Template)]
#[template(path = "make_schedule.html")]
struct MakeScheduleTemplate;

pub async fn make_schedule() -> impl IntoResponse {

    let template = MakeScheduleTemplate {};
    HtmlTemplate(template)
    
}

//-------------------------------------------------------------
// REVIEWNOTES:

#[derive(Template)]
#[template(path = "review_notes.html")]
struct ReviewNotesTemplate;

pub async fn review_notes() -> impl IntoResponse {

    let template = ReviewNotesTemplate {};
    HtmlTemplate(template)
}

//-----------------------------------------------------------
// REVIEWTREATMENTPLAN:
#[derive(Template)]
#[template(path = "view_treatment_plan.html")]
struct ViewTreatmentPlanTemplate;

pub async fn view_tp() -> impl IntoResponse {

    String::from("Who cares? I will be gone soon. I am a meditation on impermanence.");
    let template = ViewTreatmentPlanTemplate {};
    HtmlTemplate(template)
}

//-----------------------------------------------------------
// CREATEANDPOSTTREATMENTPLAN:
#[derive(Deserialize, Debug)]
pub struct TreatmentPlanNoDate {
    client: i32,
    author: i32,
    }

pub async fn post_tp(State(pool): State<Arc<AppState>>, Form(tp): Form<TreatmentPlanNoDate> ) -> impl IntoResponse {
    let today:chrono::NaiveDate = chrono::Utc::now().date_naive();
    let go_id:Result<i32, _> = sqlx::query_scalar(
        r#"
          insert into treatment_plans(client, author, date_created) values($1, $2, $3) returning tp_id   
        "#)
        .bind(tp.client)
        .bind(tp.author)
        .bind(today)
        .fetch_one(&pool.pool)
        .await;

    match go_id {
        Ok(go_id) => return String::from("plan number: ") + go_id.to_string().as_str(),
        Err(e) => return e.to_string()
    }
}
#[derive(Template)]
#[template(path = "treatmentplan.html")]
struct TreatmentPlanTemplate;

pub async fn create_tp() -> impl IntoResponse {
    let template = TreatmentPlanTemplate {};
    HtmlTemplate(template)
    // webpage has a "How many goals are you creating?" dialogue
    // user inputs a number, hits enter, which is a post request to create_goals
}

//-------------------------------------------------


//--------------------------------------------------------------
//SESSIONNOTE:

#[derive(Template)]
#[template(path = "sessionnote.html")]
struct SessionNoteTemplate;

pub async fn session_note() -> impl IntoResponse {
    let template = SessionNoteTemplate {};
    HtmlTemplate(template)
}

//-------------------------------------------------------------
//SCHEDULE:

#[derive(Template)]
#[template(path = "schedule.html")]
struct ScheduleTemplate;


pub async fn schedule() -> impl IntoResponse {
    let template = ScheduleTemplate {};
    HtmlTemplate(template)
}

//-----------------------------------------------------------------
// TODOLIST: 

#[derive(Template)]
#[template(path = "todo-list.html")]
struct TodoList {
    todos: Vec<String>,
}
//input for the Todo struct that's in the AppState
#[derive(Deserialize)]
pub struct Todo{
    todo: String,
}

pub async fn add_todo(
    State(state): State<Arc<AppState>>,
    Form(todo): Form<Todo>,
) -> impl IntoResponse {
    let mut lock = state.todos.todos.lock().unwrap();
    lock.push(todo.todo);
 
    let template = TodoList {
        todos: lock.clone(),
    };
 
    HtmlTemplate(template)
}


pub async fn hello_from_the_server() -> &'static str {
    "Hello from the server!"
}

pub async fn index() -> impl IntoResponse {
    (StatusCode::OK, "Homepage")
}


struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
    where
        T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", err)
            )
            .into_response(),
        }
    }
}


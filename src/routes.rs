use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Form, extract::State};
use chrono;
use recursive::recursive;
use serde::Deserialize;

use crate::app::AppState;
use crate::login::{self, generate_csrf_token};
use crate::login::{Backend, Role, User};

#[derive(Template, sqlx::FromRow)]
#[template(path = "error_template.html")]
struct ErrorTemplate {
  error: String,
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
        format!("Failed to render template: {}", err),
      )
        .into_response(),
    }
  }
}
//-----------------------------------------------------------
// GETMYID:
#[derive(Template)]
#[template(path = "get_my_id.html")]
pub struct GetMyIdTemplate {
  id: i32,
}

pub async fn get_my_id(auth_session: AuthSession) -> impl IntoResponse {
  let id: i32 = auth_session.user.unwrap().id;

  let template = GetMyIdTemplate { id };
  HtmlTemplate(template)
}

//-----------------------------------------------------------
// POSTGOAL:

#[derive(Deserialize, sqlx::FromRow, Debug)]
pub struct PartialGoal {
  tp_id: i32,
  data_type: String,
  goal_name: String,
  teaching_procedures: String,
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

pub async fn post_goal(State(pool): State<AppState>, Form(goal): Form<PartialGoal>) -> String {
  let full_goal = Goal {
    go_id: 0,
    tp_id: goal.tp_id,
    is_active: true,
    data_type: goal.data_type,
    goal_name: goal.goal_name,
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
    Ok(_) => String::from("Success"),
    Err(e) => e.to_string(),
  }
}
//-----------------------------------------------------------
//CREATEGOALS:
#[derive(Template)]
#[template(path = "goal-list.html")]
pub struct Goals {
  pub goals: Vec<String>,
}

#[recursive]
fn create_goals(num: &mut u32) -> Vec<String> {
  let mut ret_vec = vec![];
  while *num != 0 {
    ret_vec.push(String::from("foo"));
    *num -= 1;
  }
  ret_vec
}

#[derive(Deserialize)]
pub struct GoalNumbers {
  pub goal_numbers: u32,
}
pub async fn create_goal(Form(mut num): Form<GoalNumbers>) -> impl IntoResponse {
  let goals_vec = create_goals(&mut num.goal_numbers);

  let template = Goals { goals: goals_vec };
  HtmlTemplate(template)
}

//-----------------------------------------------------------
// VIEWGOALS:

#[derive(Template, sqlx::FromRow, Deserialize, serde::Serialize)]
#[template(path = "view_goals.html")]
pub struct GoalsTemplate {
  pub goals: Vec<Goal>,
  pub user: Option<User>,
}

#[derive(Deserialize)]
pub struct TreatmentPlanId {
  pub tp_id: u32,
  pub user: Option<User>,
}

pub async fn view_goals(
  State(pool): State<AppState>,
  auth_session: AuthSession,
  Form(num): Form<TreatmentPlanId>,
) -> impl IntoResponse {
  let num = num.tp_id as i32;
  let rows: Result<Vec<_>, sqlx::Error> = sqlx::query_as(
    "
        select * from goals where tp_id = $1
    ",
  )
  .bind(num)
  .fetch_all(&pool.pool)
  .await;

  match rows {
    Ok(goals) => {
      if goals.is_empty() {
        let goal = Goal {
          go_id: 420,
          tp_id: 69,
          is_active: false,
          data_type: String::from("error is:"),
          goal_name: "You typed in the wrong treatment plan id".to_string(),
          teaching_procedures: String::new(),
          created_at: chrono::NaiveDate::from_ymd_opt(2024, 12, 12).unwrap(),
        };
        let template = GoalsTemplate {
          goals: vec![goal],
          user: auth_session.user,
        };
        HtmlTemplate(template)
      } else {
        let template = GoalsTemplate {
          goals,
          user: auth_session.user,
        };
        HtmlTemplate(template)
      }
    }
    Err(e) => {
      let goal = Goal {
        go_id: 420,
        tp_id: 69,
        is_active: false,
        data_type: String::from("error is:"),
        goal_name: e.to_string(),
        teaching_procedures: String::new(),
        created_at: chrono::NaiveDate::from_ymd_opt(2024, 12, 12).unwrap(),
      };
      let template = GoalsTemplate {
        goals: vec![goal],
        user: auth_session.user,
      };
      HtmlTemplate(template)
    }
  }
}

//------------------------------------------------------------
//CONSULTANTSCHEDULESCREEN:

#[derive(Template)]
#[template(path = "cons_sched_base.html")]
struct ConsSchedBaseTemplate {
  pub user: Option<User>,
}

pub async fn schedules(auth_session: AuthSession) -> impl IntoResponse {
  let template = ConsSchedBaseTemplate {
    user: auth_session.user,
  };
  HtmlTemplate(template)
}
//------------------------------------------------------------
// MAKESCHEDULE:
#[derive(Template)]
#[template(path = "make_schedule.html")]
struct MakeScheduleTemplate {
  pub user: Option<User>,
  pub csrf_token: String,
}

pub async fn make_schedule(auth_session: AuthSession) -> impl IntoResponse {
  let token = generate_csrf_token();
  auth_session
    .session
    .insert("csrf_token", &token)
    .await
    .unwrap();
  let template = MakeScheduleTemplate {
    user: auth_session.user,
    csrf_token: token,
  };
  HtmlTemplate(template)
}

//-------------------------------------------------------------
// VIEWSCHEDULE:
#[derive(Template)]
#[template(path = "view_schedules.html")]
struct ViewSchedulesTemplate {
  user: Option<User>,
}

pub async fn view_schedules(auth_session: AuthSession) -> impl IntoResponse {
  let template = ViewSchedulesTemplate {
    user: auth_session.user,
  };
  HtmlTemplate(template)
}
//-------------------------------------------------------------
// POSTCONSSCHEDULE:
#[derive(Clone, Debug, Deserialize, serde::Serialize)]
pub struct ConsAppt {
  co_id: i32,
  cl_id: i32,
  appt_date: chrono::NaiveDate,
  appt_time: chrono::NaiveTime,
  parent_training: bool,
}

pub async fn post_cons_schedule(
  auth_session: AuthSession,
  State(pool): State<AppState>,
  Form(sched): Form<ConsAppt>,
) -> impl IntoResponse {
  let appt_id:Result<i32, _> = sqlx::query_scalar(
    r#"
      insert into consschedule(co_id, cl_id, appt_date, appt_time, parent_training) vales ($1, $2, $3, $4, $5) returning appt_id
    "#
  )
  .bind(sched.co_id)
  .bind(sched.cl_id)
  .bind(sched.appt_date)
  .bind(sched.appt_time)
  .bind(sched.parent_training)
  .fetch_one(&pool.pool)
  .await;
  match appt_id {
    Ok(appt_id) => String::from("Schedule ID number: ") + appt_id.to_string().as_str(),
    Err(e) => e.to_string(),
  }
}

//-------------------------------------------------------------
// POSTTECHSCHEDULE:
#[derive(Deserialize, serde::Serialize, Clone, Debug)]
pub struct TechAppt {
  pub co_id: i32,
  pub cl_id: i32,
  pub te_id: i32,
  pub appt_date: chrono::NaiveDate,
  pub appt_time: chrono::NaiveTime,
}

pub async fn post_tech_schedule(
  auth_session: AuthSession,
  State(pool): State<AppState>,
  Form(sched): Form<TechAppt>,
) -> impl IntoResponse {
  let appt_id:Result<i32, _> = sqlx::query_scalar(
        r#"
          insert into techschedule(co_id, cl_id, te_id, appt_date, appt_time) values($1, $2, $3, $4, $5) returning appt_id   
        "#)
        .bind(sched.co_id)
        .bind(sched.cl_id)
        .bind(sched.te_id)
        .bind(sched.appt_date)
        .bind(sched.appt_time)
        .fetch_one(&pool.pool)
        .await;

  match appt_id {
    Ok(appt_id) => String::from("Schedule ID number: ") + appt_id.to_string().as_str(),
    Err(e) => e.to_string(),
  }
}
//-------------------------------------------------------------
// REVIEWNOTES:

#[derive(Template)]
#[template(path = "review_notes.html")]
struct ReviewNotesTemplate {
  pub user: Option<User>,
}

pub async fn review_notes(auth_session: AuthSession) -> impl IntoResponse {
  let template = ReviewNotesTemplate {
    user: auth_session.user,
  };
  HtmlTemplate(template)
}

//-----------------------------------------------------------
// REVIEWTREATMENTPLAN:
#[derive(Template)]
#[template(path = "view_treatment_plan.html")]
struct ViewTreatmentPlanTemplate {
  pub user: Option<User>,
  pub t_p: Vec<TreatmentPlan>,
}

#[derive(Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct TreatmentPlan {
  tp_id: i32,
  client: i32,
  author: i32,
  date_created: chrono::NaiveDate,
  client_name: String,
  author_name: String,
  is_active: bool,
}

pub async fn view_tp(auth_session: AuthSession, State(pool): State<AppState>) -> impl IntoResponse {
  let _ = String::from("Who cares? I will be gone soon. I am a meditation on impermanence.");
  let id: i32 = auth_session.user.clone().unwrap().id;
  println!("id is: {:#?}", id);
  let t_p = sqlx::query_as::<_, TreatmentPlan>("select * from treatment_plans where tp_id = $1")
    .bind(id)
    .fetch_all(&pool.pool)
    .await;

  let tp = match t_p {
    Ok(tp) => tp,
    Err(e) => {
      println!("error: {:#?}", e.to_string());
      vec![]
    }
  };

  let template = ViewTreatmentPlanTemplate {
    user: auth_session.user,
    t_p: tp,
  };
  HtmlTemplate(template)
}

//-----------------------------------------------------------
// CREATETREATMENTPLAN:
#[derive(Template)]
#[template(path = "create_treatment_plan.html")]
struct CreateTreatmentPlanTemplate {
  pub user: Option<User>,
  pub id: i32,
  pub msg: String,
}

pub async fn create_tp(auth_session: AuthSession) -> impl IntoResponse {
  let template = CreateTreatmentPlanTemplate {
    user: auth_session.user.clone(),
    id: auth_session.user.unwrap().id,
    msg: String::new(),
  };
  HtmlTemplate(template)
}

//-----------------------------------------------------------
// POSTTREATMENTPLAN:
#[derive(Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct InsertTreatmentPlan {
  client: i32,
  author_id: i32,
  client_name: String,
  author_name: String,
}
pub async fn post_tp(
  State(pool): State<AppState>,
  Form(tp): Form<InsertTreatmentPlan>,
) -> impl IntoResponse {
  let today: chrono::NaiveDate = chrono::Utc::now().date_naive();
  let go_id:Result<i32, _> = sqlx::query_scalar(
        r#"
          insert into treatment_plans(client, author, date_created, client_name, author_name) values($1, $2, $3, $4, $5) returning tp_id   
        "#)
        .bind(tp.client)
        .bind(tp.author_id)
        .bind(today)
        .bind(tp.client_name)
        .bind(tp.author_name)
        .fetch_one(&pool.pool)
        .await;

  match go_id {
    Ok(go_id) => String::from("Plan ID number: ") + go_id.to_string().as_str(),
    Err(e) => e.to_string(),
  }
}
#[derive(Template)]
#[template(path = "treatment_plan_base.html")]
struct TreatmentPlanTemplate {
  pub user: Option<User>,
}

pub async fn tp(auth_session: AuthSession) -> impl IntoResponse {
  let template = TreatmentPlanTemplate {
    user: auth_session.user,
  };
  HtmlTemplate(template)
  // webpage has a "How many goals are you creating?" dialogue
  // user inputs a number, hits enter, which is a post request to create_goals
}

//-------------------------------------------------
// VIEWCLIENTS:
#[derive(Template)]
#[template(path = "view_clients.html")]
struct ClientsTemplate {
  pub user: Option<User>,
  pub clients: Vec<Clients>,
}
#[derive(Clone, Debug, Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct Clients {
  pub cl_id: i32,
  pub cl_f_name: String,
  pub cl_l_name: String,
}

pub async fn view_clients(
  auth_session: AuthSession,
  State(pool): State<AppState>,
) -> impl IntoResponse {
  let mut id: i32 = auth_session.clone().user.unwrap().id;
  let user = auth_session.user;
  let co_id: Result<i32, _> = sqlx::query_scalar("select co_id from consultants where u_id = $1")
    .bind(id)
    .fetch_one(&pool.pool)
    .await;

  match co_id {
    Ok(c_id) => id = c_id,
    Err(e) => {
      let template = ErrorTemplate {
        error: e.to_string(),
      };
      return HtmlTemplate(template).into_response();
    }
  }

  let query: Result<Vec<Clients>, _> =
    sqlx::query_as("select cl_id, cl_f_name, cl_l_name from clients where co_id = $1")
      .bind(id)
      .fetch_all(&pool.pool)
      .await;

  match query {
    Ok(clients) => {
      let template = ClientsTemplate { user, clients };
      HtmlTemplate(template).into_response()
    }
    Err(e) => {
      let template = ErrorTemplate {
        error: e.to_string(),
      };
      HtmlTemplate(template).into_response()
    }
  }
}

//--------------------------------------------------------------
//SESSIONNOTE:

#[derive(Template)]
#[template(path = "sessionnote.html")]
struct SessionNoteTemplate {
  pub user: Option<User>,
}

pub async fn session_note(auth_session: AuthSession) -> impl IntoResponse {
  let template = SessionNoteTemplate {
    user: auth_session.user,
  };
  HtmlTemplate(template)
}

//-------------------------------------------------------------
//DAILYSCHEDULE:

#[derive(Template)]
#[template(path = "schedule.html")]
struct ScheduleTemplate {
  pub user: Option<User>,
}

pub async fn schedule(auth_session: AuthSession) -> impl IntoResponse {
  let template = ScheduleTemplate {
    user: auth_session.user,
  };
  HtmlTemplate(template)
}
//------------------------------------------
// REGISTER USER:
// this will be removed in prod
#[derive(Deserialize)]
pub struct RegisterForm {
  username: String,
  password: String,
}

pub async fn register(
  State(state): State<AppState>,
  Form(data): Form<RegisterForm>,
) -> impl IntoResponse {
  let hash = login::hash_password(&data.password);

  let result = sqlx::query("Insert into users (username, password_hash) values ($1, $2)")
    .bind(&data.username)
    .bind(hash)
    .execute(&state.pool)
    .await;

  match result {
    Ok(_) => "user created",
    Err(e) => {
      println!("{:#?}", e.to_string());
      "error logged to console"
    }
  }
}
//----------------------------------------------
// LOGIN USER GET:

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
  pub user: Option<User>,
  pub csrf_token: String,
}

pub async fn get_login(auth_session: AuthSession) -> impl IntoResponse {
  let token = generate_csrf_token();
  auth_session
    .session
    .insert("csrf_token", &token)
    .await
    .unwrap();
  let template = LoginTemplate {
    user: auth_session.user,
    csrf_token: token,
  };
  HtmlTemplate(template)
}

//-----------------------------------------------
// LOGIN USER POST:

#[derive(Clone, Deserialize)]
pub struct LoginForm {
  username: String,
  password: String,
  csrf_token: String,
}

type AuthSession = axum_login::AuthSession<Backend>;

pub async fn login(
  mut auth_session: AuthSession,
  // session: Session,
  Form(creds): Form<LoginForm>,
) -> impl IntoResponse {
  let session_token: Option<String> = auth_session.session.get("csrf_token").await.unwrap();
  if session_token.as_deref() != Some(&creds.csrf_token) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let auth_result = auth_session
    .authenticate((creds.username, creds.password))
    .await;

  let user = match auth_result {
    Ok(Some(user)) => user,
    Ok(_none) => {
      println!("inval crdensh");
      return StatusCode::UNAUTHORIZED.into_response();
    }
    Err(e) => {
      println!("{:#?}", e);
      return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
  };
  // println!("skip login");
  match auth_session.login(&user).await {
    Ok(_) => Redirect::to("/schedule").into_response(),
    Err(e) => {
      println!("{:#?}", e);
      "failure".into_response()
    }
  }
}

//-----------------------------------------------
// LOGOUT USER:
//
pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
  auth_session.logout().await.unwrap();
  println!("Logged out!");
  Redirect::to("/login").into_response()
}
//--------------------------------------x-----------------
// CHECK: GETS USER ID AND DISPLAYS ROLE
pub async fn get_id(auth_session: AuthSession) -> impl IntoResponse {
  let user = auth_session.user.unwrap();
  let user_id = user.id.to_string();
  let role = user.role;
  println!("Role is: {:#?}", role);

  user_id.into_response()
}

//--------------------------------------x-----------------
// HELPER: ADMIN ONLY, CONSULTANT & ADMIN

pub async fn admin_check(user: &User) -> Result<(), StatusCode> {
  match user.role {
    Role::Admin => Ok(()),
    _ => Err(StatusCode::FORBIDDEN),
  }
}

pub async fn consultant_check(user: &User) -> Result<(), StatusCode> {
  match user.role {
    Role::Tech => Err(StatusCode::FORBIDDEN),
    _ => Ok(()),
  }
}

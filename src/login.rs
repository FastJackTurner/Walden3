use argon2::{
  Argon2,
  password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    rand_core::{OsRng, RngCore},
  },
};
use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;

#[derive(Clone, Deserialize, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum Role {
  Tech,
  Consultant,
  Admin,
}

impl Role {
  pub fn can_access(&self, required: &Role) -> bool {
    self >= required
  }
}

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct User {
  pub id: i32,
  pub username: String,
  pub password_hash: String,
  pub role: Role,
  pub f_name: String,
  pub l_name: String,
}

impl User {
  pub fn can_access(&self, role: Role) -> bool {
    self.role >= role
  }
}

impl AuthUser for User {
  type Id = i32;

  fn id(&self) -> Self::Id {
    self.id
  }

  fn session_auth_hash(&self) -> &[u8] {
    self.password_hash.as_bytes()
  }
}

#[derive(Clone)]
pub struct Backend {
  pub db: PgPool,
}

// #[derive(Clone)]
// pub struct Credentials {
//     user_pass: (String, String), //username, password
// }

impl AuthnBackend for Backend {
  type User = User;
  type Credentials = (String, String);
  type Error = sqlx::Error;

  async fn authenticate(
    &self,
    credentials: Self::Credentials,
  ) -> Result<Option<Self::User>, Self::Error> {
    let user = sqlx::query_as::<_, User>(
      "Select id, username, password_hash, role, f_name, l_name from users where username = $1",
    )
    .bind(credentials.0)
    .fetch_optional(&self.db)
    .await?;

    if let Some(user) = user {
      println!("user from DB: {:#?}", user);
      if verify_password(&credentials.1, &user.password_hash) {
        return Ok(Some(user));
      }
    }
    Ok(None)
  }

  async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
    sqlx::query_as::<_, User>(
      "select id, username, password_hash, role, f_name, l_name from users where id = $1",
    )
    .bind(user_id)
    .fetch_optional(&self.db)
    .await
  }
}

pub fn hash_password(password: &str) -> String {
  let salt = SaltString::generate(&mut OsRng);
  Argon2::default()
    .hash_password(password.as_bytes(), &salt)
    .unwrap()
    .to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
  let parsed = PasswordHash::new(hash).unwrap();
  Argon2::default()
    .verify_password(password.as_bytes(), &parsed)
    .is_ok()
}

pub fn generate_csrf_token() -> String {
  let mut bytes = [0u8; 32];
  OsRng.fill_bytes(&mut bytes);
  bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

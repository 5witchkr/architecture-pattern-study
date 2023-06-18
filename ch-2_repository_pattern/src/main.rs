
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use diesel::prelude::*;
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection, RunQueryDsl,
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

table! {
    users (id) {
        id -> Integer,
        name -> Text,
        hair_color -> Nullable<Text>,
    }
}

#[derive(serde::Serialize, Selectable, Queryable)]
pub struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
}

#[derive(serde::Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    name: String,
    hair_color: Option<String>,
}

// Repository trait
#[async_trait]
pub trait UserRepository {
    async fn create_user(&self, new_user: NewUser) -> Result<User, diesel::result::Error>;
    async fn list_users(&self) -> Result<Vec<User>, diesel::result::Error>;
}

// Repository implementation
#[derive(Clone)]
pub struct DieselUserRepository {
    pool: Pool,
}

#[async_trait]
impl UserRepository for DieselUserRepository {
    async fn create_user(&self, new_user: NewUser) -> Result<User, diesel::result::Error> {
        let mut conn = self.pool.get().await.unwrap();
        let res = diesel::insert_into(users::table)
            .values(new_user)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await?;
        Ok(res)
    }

    async fn list_users(&self) -> Result<Vec<User>, diesel::result::Error> {
        let mut conn = self.pool.get().await.unwrap();
        let res = users::table
            .select(User::as_select())
            .load(&mut conn)
            .await?;
        Ok(res)
    }
}

type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_diesel_async_postgres=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").unwrap();

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let pool = bb8::Pool::builder().build(config).await.unwrap();
    let user_repository = DieselUserRepository { pool };

    let app = Router::new()
    .route("/user/list", get(list_users::<DieselUserRepository>))
    .route("/user/create", post(create_user::<DieselUserRepository>))
    .with_state(user_repository);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn create_user<R>(
    State(user_repository): State<R>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, (StatusCode, String)>
where
    R: UserRepository,
{
    let res = user_repository.create_user(new_user).await.map_err(internal_error)?;
    Ok(Json(res))
}

struct DatabaseConnection(
    bb8::PooledConnection<'static, AsyncDieselConnectionManager<AsyncPgConnection>>,
);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = Pool::from_ref(state);
        let conn = pool.get_owned().await.map_err(internal_error)?;
        Ok(Self(conn))
    }
}

async fn list_users<R>(
    State(user_repository): State<R>,
) -> Result<Json<Vec<User>>, (StatusCode, String)>
where
    R: UserRepository,
{
    let res = user_repository.list_users().await.map_err(internal_error)?;
    Ok(Json(res))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
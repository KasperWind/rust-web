use std::sync::Arc;

use axum::{Router, response::Json, routing::get, extract::State};
use serde_json::{Value, json};
use toasty::stmt::Id;

#[derive(Debug, toasty::Model)]
struct User {
    #[key]
    #[auto]
    id: Id<Self>,

    name: String,

    #[unique]
    email: String,

    #[has_many]
    todos: toasty::HasMany<Todo>,
    moto: Option<String>,
}

#[derive(Debug, toasty::Model)]
struct Todo {
    #[key]
    #[auto]
    id: Id<Self>,

    #[index]
    user_id: Id<User>,

    #[belongs_to(key = user_id, references = id)]
    user: toasty::BelongsTo<User>,
    
    title: String,
}

#[derive(Debug)]
struct AppState {
    db : toasty::Db,
}

#[tokio::main]
async fn main() -> toasty::Result<()> {
    let db = toasty::Db::builder()
        .register::<User>()
        .register::<Todo>()
        .connect("sqlite:mydb.db")
        .await?;

    let state = Arc::new( AppState { db });

    let app = Router::new()
        .route("/", get(|| async { 
            println!("route `/`");
            "Hello Worlds!" 
        }))
        .route("/users", get(get_all_users))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
#[derive(Debug, serde::Serialize)]
struct UserJson {
    name: String,
    email: String,
    moto: Option<String>,
}

async fn get_all_users(
    State(state) : State<Arc<AppState>>,
) -> Json<Value> {
    println!("route `/users`");

    let users = User::all().all(&state.db).await;
    match users {
        Ok(users) => {
            let users: Vec<User> = users.collect().await.unwrap();
            let users: Vec<UserJson> = users.iter().map(|f| { 
                UserJson { name: f.name.clone(), email: f.email.clone(), moto: f.moto.clone() }
            } ).collect();
            Json(json!(users))
        },
        Err(err) => {
            let s = err.to_string();
            eprintln!("could not get users, err: {:?}", err);
            Json(json!({"Error" : "could not get users", "Details" : s}))
        },
    }
}

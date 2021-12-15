use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder,Error, HttpRequest};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, PgPool, Pool};
use dotenv::dotenv;
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    username: String,
    fullname: String
}

#[post("/adduser")]
async fn add_user(req: web::Json<User>, db_pool: web::Data<PgPool>) -> impl Responder {
    println!("POST: /adduser");

    let user_new = req.into_inner();
    let tx = db_pool.get_ref();

    let _row = query!(
        r#"
        INSERT INTO users (username, fullname) VALUES
        ($1, $2)
        "#,
        user_new.username,
        user_new.fullname,
    )
    .execute(tx)
    .await;

    let res = match _row {
        Err(_) => {
            json!({
                "status": "error",
                "message": "User already exists!"
            })
        },
        Ok(1) => {
            json!({
                "status": "ok",
                "message": "User created Successfully!"
            })
        },
        Ok(_) => {
            json!({
                "status": "error",
                "message": "Could not create the user! Please try again!"
            })
        }
    };

    HttpResponse::Ok().json(res)
}

#[get("/getuser/{name}")]
async fn get_user(name: web::Path<String>, db_pool: web::Data<PgPool>) -> impl Responder {
    println!("GET: /getuser/[name]");

    let new_pool = db_pool.get_ref();
    let user_name = name.to_string();

    let _row = query_as!(User,
        r#"
        SELECT username, fullname from users WHERE username = $1
        "#,
        user_name
    )
    .fetch_one(new_pool)
    .await;

    let res = match _row {
        Err(_) => {
            json!({
                "error": "User not found!"
            })
        },
        Ok(user) => {
            json!(
                user
            )
        }
    };

    HttpResponse::Ok().json(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_pool = make_db_pool().await;
    
    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .service(add_user)
            .service(get_user)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

pub async fn make_db_pool() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    println!("Connected to database: {}", db_url);
    Pool::new(&db_url).await.unwrap()
}
use actix_web::{get, post, put, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{query, query_as, PgPool, Pool};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    username: String,
    fullname: String,
}

#[post("/adduser")]
async fn add_user(req: web::Json<User>, db_pool: web::Data<PgPool>) -> impl Responder {
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
        }
        Ok(1) => {
            json!({
                "status": "ok",
                "message": "User created Successfully!"
            })
        }
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
    let new_pool = db_pool.get_ref();
    let user_name = name.to_string();

    let _row = query_as!(
        User,
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
        }
        Ok(user) => {
            json!(user)
        }
    };

    HttpResponse::Ok().json(res)
}

#[put("/updateuser")]
async fn update_user(req: web::Json<User>, db_pool: web::Data<PgPool>) -> impl Responder {
    let new_pool = db_pool.get_ref();
    let user_name = req.username.to_string();
    let full_name = req.fullname.to_string();

    let _row = query!(
        r#"
        UPDATE users set fullname = $1 where username = $2
        "#,
        full_name,
        user_name
    )
    .execute(new_pool)
    .await;

    let res = match _row {
        Err(_) => {
            json!({
                "status": "error",
                "message": "Could not update the user details. Please try again later."
            })
        }
        Ok(1) => {
            json!({
                "status": "ok",
                "message": "User details updated Successfully!"
            })
        }
        Ok(_) => {
            json!({
                "status": "error",
                "message": "Could not update the user details. Please try again later."
            })
        }
    };

    HttpResponse::Ok().json(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_pool = make_db_pool().await;
    println!("Server Started at port: 8080");

    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .service(add_user)
            .service(get_user)
            .service(update_user)
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

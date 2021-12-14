use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder,Error, HttpRequest};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, PgPool, Pool};
use dotenv::dotenv;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    username: String,
    fullname: String
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[post("/adduser")]
async fn add_user(req: web::Json<User>, db_pool: web::Data<PgPool>) -> impl Responder {

    let user_new = req.into_inner();
    let tx = db_pool.get_ref();

    let row = query_as!(User,
        r#"
        INSERT INTO users (username, fullname) VALUES
        ($1, $2) returning username, fullname
        "#,
        user_new.username,
        user_new.fullname,
    )
    .fetch_one(tx)
    .await;

    let user: User = User {
        username: user_new.username,
        fullname: user_new.fullname
    };

    HttpResponse::Ok().json(user)
}

#[get("/getuser/{name}")]
async fn get_user(name: web::Path<String>, db_pool: web::Data<PgPool>) -> impl Responder {

    let tx = db_pool.get_ref();
    let user_name = name.to_string();

    println!("{}", user_name);

    let row = query_as!(User,
        r#"
        SELECT username, fullname from users WHERE username = $1
        "#,
        user_name
    )
    .fetch_one(tx)
    .await;

    println!("{:?}", row);

    HttpResponse::Ok()
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
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{query, query_as, PgPool, Pool};

#[derive(Serialize, Deserialize, Debug)]
struct UserBooks {
    bookname: String,
    isbn: String,
    authorname: String,
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Book {
    bookname: String,
    isbn: String,
    authorname: String,
}

#[get("/getallbooks")]
async fn get_all_books(db_pool: web::Data<PgPool>) -> impl Responder {
    let tx = db_pool.get_ref();

    let _row = query_as!(
        Book,
        r#"
        SELECT bookname, isbn, authorname from books
        "#
    )
    .fetch_all(tx)
    .await;

    let res = match _row {
        Err(_) => {
            json!({
                "error": "User not found!"
            })
        }
        Ok(books) => {
            json!(books)
        }
    };

    HttpResponse::Ok().json(res)
}

#[get("/getbooks/{name}")]
async fn get_user_books(name: web::Path<String>, db_pool: web::Data<PgPool>) -> impl Responder {
    let new_pool = db_pool.get_ref();
    let user_name = name.to_string();

    let _row = query_as!(
        Book,
        r#"
        SELECT bookname, isbn, authorname from books WHERE username = $1
        "#,
        user_name
    )
    .fetch_all(new_pool)
    .await;

    let res = match _row {
        Err(_) => {
            json!({
                "error": "User not found!"
            })
        }
        Ok(books) => {
            json!(books)
        }
    };

    HttpResponse::Ok().json(res)
}

#[post("/addbook")]
async fn add_book(req: web::Json<UserBooks>, db_pool: web::Data<PgPool>) -> impl Responder {
    let book_new = req.into_inner();
    let tx = db_pool.get_ref();

    let _row = query!(
        r#"
        INSERT INTO books (bookname, isbn, authorname, username) VALUES
        ($1, $2, $3, $4)
        "#,
        book_new.bookname,
        book_new.isbn,
        book_new.authorname,
        book_new.username,
    )
    .execute(tx)
    .await;

    let res = match _row {
        Err(_) => {
            json!({
                "status": "error",
                "message": "Book already exists!"
            })
        }
        Ok(1) => {
            json!({
                "status": "ok",
                "message": "Book added Successfully!"
            })
        }
        Ok(_) => {
            json!({
                "status": "error",
                "message": "Could not add the book! Please try again!"
            })
        }
    };

    HttpResponse::Ok().json(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_pool = make_db_pool().await;
    println!("Server Started at port: 8081");

    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .service(get_all_books)
            .service(get_user_books)
            .service(add_book)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}

pub async fn make_db_pool() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    println!("Connected to database: {}", db_url);
    Pool::new(&db_url).await.unwrap()
}

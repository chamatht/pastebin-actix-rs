#[macro_use] extern crate serde_json;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

use actix_web::{
    get, post, middleware, web, App, HttpResponse, HttpServer, http::header
};
use serde::{Deserialize};
use tera::Tera;
use tokio_postgres::{Client, NoTls};

#[derive(Deserialize)]
struct FormData {
    title: String,
    text: String
}

#[post("/pastedata")]
async fn form(form: web::Form<FormData>,
              dbl: web::Data<Client>) -> HttpResponse
{
    let stmt = dbl.prepare(
        "INSERT INTO pastes(title,text) VALUES($1,$2) RETURNING uid;")
                .await.unwrap();
    let uid:i64 = dbl.query_one(&stmt, &[&form.title, &form.text])
                    .await.unwrap().get(0);
    let location = "/tx/".to_string() + uid.to_string().as_str();
    HttpResponse::MovedPermanently()
        .header(header::LOCATION,location)
        .finish().into_body()
}

#[post("/deltx/{uid}")]
async fn delete_paste(uid: web::Path<i64>,
                       dbl: web::Data<Client>) -> HttpResponse
{
    let stmt
        = dbl.prepare("DELETE FROM pastes WHERE uid=$1;").await.unwrap();
    dbl.execute(&stmt, &[uid.as_ref()]).await.unwrap();

    HttpResponse::MovedPermanently()
        .header(header::LOCATION,"/browse")
        .finish().into_body()
}

#[get("/tx/{uid}")]
async fn display_paste(uid: web::Path<i64>,
                       tera: web::Data<Tera>,
                       dbl: web::Data<Client>) -> HttpResponse
{
    let stmt
        = dbl.prepare("SELECT title,text FROM pastes WHERE uid=$1;").await.unwrap();
    let row = dbl.query_one(&stmt, &[uid.as_ref()]).await.unwrap();
    let title: String = row.get(0);
    let text: String = row.get(1);

    let mut ctx = tera::Context::new();
    ctx.insert("title", &title);
    ctx.insert("text", &text);
    ctx.insert("uid", uid.as_ref());
    let body = tera.render("paste.html", &ctx).unwrap();

    HttpResponse::Ok().body(body)
}

#[get("/")]
async fn index(t: web::Data<Tera>) -> HttpResponse {
    let mut ctx = tera::Context::new();
    ctx.insert("name", "");
    let body = t.render("index.html", &ctx).unwrap();

    HttpResponse::Ok().body(body)
}

#[get("/browse")]
async fn browse(tera: web::Data<Tera>, dbl: web::Data<Client>) -> HttpResponse {
    let stmt
        = dbl.prepare("SELECT uid,title FROM pastes;").await.unwrap();
    let rows
        = dbl.query(&stmt, &[]).await.unwrap();
    let tts: Vec<(i64, String)>
        = rows.iter().map(|r| (r.get(0),r.get(1)))
            .collect();

    let data = json!({"tl": tts});
    let ctx = tera::Context::from_value(data).unwrap();
    let body = tera.render("browse.html", &ctx).unwrap();

    HttpResponse::Ok().body(body)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let (ct, connection) =
        tokio_postgres::connect(
            "host=localhost dbname=pastebin user=postsql password=postsql",
            NoTls).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = &connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let dbdata = web::Data::new(ct);

    HttpServer::new(move || {
        let tt = Tera::new("templates/**/*").unwrap();
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .data(tt)
            .register_data(dbdata.clone())
            .service(form)
            .service(display_paste)
            .service(index)
            .service(browse)
            .service(delete_paste)
    })
        .bind("127.0.0.1:8080")?
        .start()
        .await
}

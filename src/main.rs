#[macro_use]
extern crate serde_json;

use actix_web::{
    get, post, middleware, web, App, HttpRequest, HttpResponse, HttpServer
};
use serde::{Deserialize};
use tera::Tera;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Deserialize)]
struct FormData {
    title: String,
    text: String
}

struct TextData {
    hm: Mutex<HashMap<String,String>>
}

#[post("/pastedata")]
async fn testform(t: web::Data<Tera>,
                  form: web::Form<FormData>,
                  tx: web::Data<TextData>) -> HttpResponse
{
    let mut ctx = tera::Context::new();
    ctx.insert("title", &form.title);
    ctx.insert("text", &form.text);
    let body = t.render("paste.html", &ctx).unwrap();
    let mut hm = tx.hm.lock().unwrap();
    hm.insert(form.title.clone(), form.text.clone());
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
async fn browse(t: web::Data<Tera>, tx: web::Data<TextData>) -> HttpResponse {
    let mut ctx = tera::Context::new();
    let hm = tx.hm.lock().unwrap();
    ctx.insert("name", hm.get("hello").unwrap());
    let body = t.render("index.html", &ctx).unwrap();

    HttpResponse::Ok().body(body)
}

#[get("/w/{name}")]
async fn dynamic_name(
    t: web::Data<Tera>,
    name: web::Path<String>) -> HttpResponse
{
    let data = json!({
        "name": name.to_string()
    });
    let ctx = tera::Context::from_value(data).unwrap();
    let body = t.render("index.html", &ctx).unwrap();

    HttpResponse::Ok().body(body)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();
    HttpServer::new(|| {
        let tt = Tera::new("templates/**/*").unwrap();
        let db = web::Data::new(TextData {
            hm: Mutex::new(HashMap::new())
        });
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .data(tt)
            .register_data(db)
            .service(testform)
            .service(index)
            .service(dynamic_name)
            .service(browse)
    })
        .bind("127.0.0.1:8080")?
        //.workers(1)
        .start()
        .await
}

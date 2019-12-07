#[macro_use] extern crate serde_json;

use actix_web::{
    get, post, middleware, web, App, HttpResponse, HttpServer
};
use serde::{Serialize,Deserialize};
use tera::Tera;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Deserialize)]
struct FormData {
    title: String,
    text: String
}

struct TextData {
    hm: Mutex<HashMap<Uuid,(String,String)>>
}

#[derive(Serialize,Deserialize)]
struct BrowseData{
    uuid:String,
    title:String,
}

#[post("/pastedata")]
async fn testform(tera: web::Data<Tera>,
                  form: web::Form<FormData>,
                  db: web::Data<TextData>) -> HttpResponse
{
    let mut ctx = tera::Context::new();
    ctx.insert("title", &form.title);
    ctx.insert("text", &form.text);
    let body = tera.render("paste.html", &ctx).unwrap();
    let mut dbl = db.hm.lock().unwrap();
    dbl.insert(Uuid::new_v4(), (form.title.clone(), form.text.clone()));

    HttpResponse::Ok().body(body)
}

#[get("/tx/{uid}")]
async fn display_paste(uid: web::Path<Uuid>,
                       tera: web::Data<Tera>,
                       db: web::Data<TextData>) -> HttpResponse
{
    let mut ctx = tera::Context::new();
    let dblocked = db.hm.lock().unwrap();
    let (title, text) = dblocked.get(&uid).unwrap();
    ctx.insert("title", &title);
    ctx.insert("text", &text);
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
async fn browse(tera: web::Data<Tera>, db: web::Data<TextData>) -> HttpResponse {
    let dbl = db.hm.lock().unwrap();
    let title_list: Vec<BrowseData> = dbl.iter()
        .map(|(u,(a,_))|
            BrowseData{ uuid: u.to_string(), title:a.to_string()} ).collect();

    let data = json!({"tl": title_list});

    let ctx = tera::Context::from_value(data).unwrap();
    let body = tera.render("browse.html", &ctx).unwrap();

    HttpResponse::Ok().body(body)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();
    let db = web::Data::new(TextData {
        hm: Mutex::new(HashMap::new())
    });

    HttpServer::new(move || {
        let tt = Tera::new("templates/**/*").unwrap();
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .data(tt)
            .register_data(db.clone())
            .service(testform)
            .service(display_paste)
            .service(index)
            .service(browse)
    })
        .bind("127.0.0.1:8080")?
        .start()
        .await
}

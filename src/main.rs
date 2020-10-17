use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use lazy_static::lazy_static;
use serde_derive::Deserialize;
use std::sync::Mutex;
use tera::{Context, Tera};

mod fake_db;
mod person;

lazy_static! {
    pub static ref TERA: Tera = Tera::new("src/templates/**").unwrap();
}

pub struct AppState {
    db: fake_db::FakeDb,
}

fn get_main() -> impl Responder {
    let ctx = Context::new();

    HttpResponse::Ok()
        .content_type("text/html")
        .body(TERA.render("index.html", &ctx).unwrap())
}

#[derive(Deserialize)]
pub struct Filter {
    partial_name: Option<String>,
}

fn get_page_persons(
    query: web::Query<Filter>,
    state: web::Data<Mutex<AppState>>,
) -> impl Responder {
    let partial_name = query.partial_name.clone().unwrap_or(String::default());
    let db = &state.lock().unwrap().db;
    let mut ctx = Context::new();

    ctx.insert("partial_name", &partial_name);

    if partial_name.as_str() != String::default() {
        ctx.insert(
            "persons",
            &db.get_persons_by_name(partial_name.as_str())
                .collect::<Vec<_>>(),
        );
    } else {
        ctx.insert("persons", &db.get_all_persons());
    }

    HttpResponse::Ok()
        .content_type("text/html")
        .body(TERA.render("persons.html", &ctx).unwrap())
}

fn invalid_resource() -> impl Responder {
    HttpResponse::NotFound()
        .content_type("text/html")
        .body("<h2>Not Found</h2>");
}

fn main() -> std::io::Result<()> {
    let address = "127.0.0.1:8080";

    println!("Server listening on http://{}", address);

    let state = web::Data::new(Mutex::new(AppState {
        db: fake_db::FakeDb::new(),
    }));

    HttpServer::new(move || {
        App::new()
            .register_data(state.clone())
            .service(web::resource("/").route(web::get().to(get_main)))
            .service(web::resource("/page/persons").route(web::get().to(get_page_persons)))
            .default_service(web::route().to(invalid_resource))
    })
    .bind(address)?
    .run()
}

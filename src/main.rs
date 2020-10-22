use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web_httpauth::extractors::basic::{BasicAuth, Config};
use lazy_static::lazy_static;
use serde_derive::Deserialize;
use std::sync::Mutex;
use tera::{Context, Tera};

mod fake_db;
mod person;
mod user;

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

/// Query Params to filter persons
#[derive(Deserialize)]
pub struct Filter {
    partial_name: Option<String>,
}

/// Query Params for the "DELETE" page
#[derive(Deserialize)]
pub struct ToDelete {
    id_list: Option<String>,
}

/// Query Params for the "INSERT" page
#[derive(Deserialize)]
struct ToInsert {
    name: Option<String>,
}

/// Query Params for the "UPDATE" page
#[derive(Deserialize)]
struct ToUpdate {
    id: Option<u32>,
    name: Option<String>,
}

fn authorize(
    auth: BasicAuth,
    state: &web::Data<Mutex<AppState>>,
    required_priviledge: user::DbPrivilege,
) -> Result<Vec<user::DbPrivilege>, String> {
    let db = &state.lock().unwrap().db;

    if let Some(user) = db.get_user_by_username(auth.user_id()) {
        if auth.password().is_some() && &user.password == auth.password().unwrap() {
            if user.privileges.contains(&required_priviledge) {
                return Ok(user.privileges.clone());
            }

            return Err(String::from("Forbidden"));
        }

        return Err(String::from("Invalid password"));
    }

    Err(String::from("User not found"))
}

fn get_page_login_with_message(message: &str) -> HttpResponse {
    let mut ctx = Context::new();

    ctx.insert("error_message", message);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(TERA.render("login.html", &ctx).unwrap())
}

fn get_page_persons(
    query: web::Query<Filter>,
    auth: BasicAuth,
    state: web::Data<Mutex<AppState>>,
) -> HttpResponse {
    match authorize(auth, &state, user::DbPrivilege::CanRead) {
        Ok(_) => {
            let partial_name = query.partial_name.clone().unwrap_or(String::default());
            let db = &state.lock().unwrap().db;
            let mut ctx = Context::new();

            ctx.insert("id_error", "");
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

            return HttpResponse::Ok()
                .content_type("text/html")
                .body(TERA.render("persons.html", &ctx).unwrap());
        }
        Err(msg) => get_page_login_with_message(&msg),
    }
}

fn delete_persons(
    query: web::Query<ToDelete>,
    state: web::Data<Mutex<AppState>>,
    auth: BasicAuth,
) -> HttpResponse {
    match authorize(auth, &state, user::DbPrivilege::CanWrite) {
        Ok(_) => {
            let db = &mut state.lock().unwrap().db;
            let mut deleted = 0;

            query
                .id_list
                .clone()
                .unwrap_or(String::default())
                .split_terminator(',')
                .for_each(|id| {
                    if db.delete(id.parse::<u32>().unwrap()) {
                        deleted += 1;
                    }
                });

            HttpResponse::Ok()
                .content_type("text/plain")
                .body(deleted.to_string())
        }
        Err(msg) => get_page_login_with_message(&msg),
    }
}

fn get_page_new_person(auth: BasicAuth, state: web::Data<Mutex<AppState>>) -> HttpResponse {
    match authorize(auth, &state, user::DbPrivilege::CanWrite) {
        Ok(privileges) => {
            let mut ctx = Context::new();

            ctx.insert(
                "can_write",
                &privileges.contains(&user::DbPrivilege::CanWrite),
            );
            ctx.insert("person_id", "");
            ctx.insert("person_name", "");
            ctx.insert("inserting", &true);

            HttpResponse::Ok()
                .content_type("text/html")
                .body(TERA.render("one_person.html", &ctx).unwrap())
        }
        Err(msg) => get_page_login_with_message(&msg),
    }
}

fn respond_with_error(error_message: &str, state: &web::Data<Mutex<AppState>>) -> HttpResponse {
    let db = &state.lock().unwrap().db;
    let mut ctx = Context::new();

    ctx.insert("id_error", error_message);
    ctx.insert("partial_name", &"");

    let person_list = db.get_all_persons();
    ctx.insert("persons", &person_list);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(TERA.render("persons.html", &ctx).unwrap())
}

fn get_page_edit_person(
    state: web::Data<Mutex<AppState>>,
    path: web::Path<(String,)>,
    auth: BasicAuth,
) -> HttpResponse {
    match authorize(auth, &state, user::DbPrivilege::CanWrite) {
        Ok(privileges) => {
            let id = path.0.as_str();
            let db = &state.lock().unwrap().db;
            let mut ctx = Context::new();

            ctx.insert(
                "can_write",
                &privileges.contains(&user::DbPrivilege::CanWrite),
            );

            if let Ok(id) = id.parse::<u32>() {
                if let Some(person) = db.get_person_by_id(id) {
                    ctx.insert("person_id", &id);
                    ctx.insert("person_name", &person.name);
                    ctx.insert("inserting", &false);

                    return HttpResponse::Ok()
                        .content_type("text/html")
                        .body(TERA.render("one_person.html", &ctx).unwrap());
                } else {
                    return respond_with_error(&format!("No person with id: {} found", id), &state);
                }
            }

            return respond_with_error(&format!("No person with id: {} found", id), &state);
        }
        Err(msg) => get_page_login_with_message(&msg),
    }
}

fn create_person(
    state: web::Data<Mutex<AppState>>,
    query: web::Query<ToInsert>,
    auth: BasicAuth,
) -> HttpResponse {
    match authorize(auth, &state, user::DbPrivilege::CanWrite) {
        Ok(_) => {
            let db = &mut state.lock().unwrap().db;
            let mut inserted = 0;

            if let Some(name) = &query.name.clone() {
                db.insert(name);
                inserted += 1;
            }

            HttpResponse::Ok()
                .content_type("text/html")
                .body(inserted.to_string())
        }
        Err(msg) => get_page_login_with_message(&msg),
    }
}

fn update_person(
    state: web::Data<Mutex<AppState>>,
    query: web::Query<ToUpdate>,
    auth: BasicAuth,
) -> HttpResponse {
    match authorize(auth, &state, user::DbPrivilege::CanWrite) {
        Ok(privileges) => {
            let db = &mut state.lock().unwrap().db;

            if let (Some(id), Some(name)) = (&query.id, &query.name) {
                return HttpResponse::Created()
                    .content_type("text/html")
                    .body(db.update(*id, name).to_string());
            }

            return HttpResponse::NotModified()
                .content_type("text/html")
                .body(0.to_string());
        }
        Err(msg) => get_page_login_with_message(&msg),
    }
}

fn get_page_login() -> HttpResponse {
    get_page_login_with_message("")
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
            .data(Config::default().realm("PersonsApplication"))
            .service(web::resource("/").route(web::get().to(get_main)))
            .service(web::resource("/page/login").route(web::get().to(get_page_login)))
            .service(web::resource("/page/persons").route(web::get().to(get_page_persons)))
            .service(web::resource("/page/new_person").route(web::get().to(get_page_new_person)))
            .service(
                web::resource("/page/edit_person/{id}").route(web::get().to(get_page_edit_person)),
            )
            .service(web::resource("/persons").route(web::delete().to(delete_persons)))
            .service(
                web::resource("/one_person")
                    .route(web::post().to(create_person))
                    .route(web::put().to(update_person)),
            )
            .default_service(web::route().to(invalid_resource))
    })
    .bind(address)?
    .run()
}

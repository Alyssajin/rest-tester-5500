use actix_cors::Cors;
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u32,
    name: String,
    hours_worked: u32,
}

struct AppState {
    users: Mutex<Vec<User>>,
    next_id: Mutex<u32>,
}

#[derive(Deserialize)]
struct UserRequest {
    name: String,
}

#[derive(Deserialize)]
struct HoursRequest {
    hours_to_add: u32,
}

async fn get_users(data: web::Data<AppState>) -> impl Responder {
    println!("GET /users - Fetching all users");
    let users = data.users.lock().unwrap();
    HttpResponse::Ok().json(&*users)
}

async fn get_user_by_id(data: web::Data<AppState>, user_id: web::Path<u32>) -> impl Responder {
    println!("GET /users/{} - Fetching user with ID: {}", user_id, user_id);
    let users = data.users.lock().unwrap();
    if let Some(user) = users.iter().find(|u| u.id == *user_id) {
        println!("User found: {:?}", user);
        HttpResponse::Ok().json(user)
    } else {
        println!("User not found");
        HttpResponse::NotFound().body("User not found")
    }
}

async fn delete_all_users(data: web::Data<AppState>) -> impl Responder {
    println!("DELETE /users - Deleting all users");
    let mut users = data.users.lock().unwrap();
    users.clear();
    let mut next_id = data.next_id.lock().unwrap();
    *next_id = 1;
    HttpResponse::Ok().json(&*users)
}

async fn add_user(user: web::Json<UserRequest>, data: web::Data<AppState>) -> impl Responder {
    println!("POST /users - Adding new user");
    let mut users = data.users.lock().unwrap();
    let mut next_id = data.next_id.lock().unwrap();
    let new_user = User {
        id: *next_id,
        name: user.name.trim().to_string(),
        hours_worked: 0,
    };
    if new_user.name.is_empty() {
        println!("Invalid name provided");
        return HttpResponse::BadRequest().body("Name is required and must be a non-empty string");
    }
    *next_id += 1;
    users.push(new_user.clone());
    println!("New user added: {:?}", new_user);
    HttpResponse::Created().json(new_user)
}

async fn update_user_by_id(data: web::Data<AppState>, user_id: web::Path<u32>, user_update: web::Json<UserRequest>) -> impl Responder {
    println!("PUT /users/{} - Updating user with ID: {}", user_id, user_id);
    let mut users = data.users.lock().unwrap();
    if let Some(user) = users.iter_mut().find(|u| u.id == *user_id) {
        if !user_update.name.trim().is_empty() {
            user.name = user_update.name.trim().to_string();
            println!("Updated user name to: {}", user.name);
        }
        HttpResponse::Ok().json(user)
    } else {
        println!("User not found");
        HttpResponse::NotFound().body("User not found")
    }
}

async fn update_user_hours(data: web::Data<AppState>, user_id: web::Path<u32>, hours_update: web::Json<HoursRequest>) -> impl Responder {
    println!("PATCH /users/{} - Updating hours for user with ID: {}", user_id, user_id);
    let mut users = data.users.lock().unwrap();
    if let Some(user) = users.iter_mut().find(|u| u.id == *user_id) {
        user.hours_worked += hours_update.hours_to_add;
        println!("Added {} hours. Total hours worked: {}", hours_update.hours_to_add, user.hours_worked);
        HttpResponse::Ok().json(user)
    } else {
        println!("User not found");
        HttpResponse::NotFound().body("User not found")
    }
}

async fn delete_user_by_id(data: web::Data<AppState>, user_id: web::Path<u32>) -> impl Responder {
    println!("DELETE /users/{} - Deleting user with ID: {}", user_id, user_id);
    let mut users = data.users.lock().unwrap();
    if let Some(pos) = users.iter().position(|u| u.id == *user_id) {
        let deleted_user = users.remove(pos);
        println!("User deleted: {:?}", deleted_user);
        HttpResponse::Ok().json(deleted_user)
    } else {
        println!("User not found");
        HttpResponse::NotFound().body("User not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = web::Data::new(AppState {
        users: Mutex::new(Vec::new()),
        next_id: Mutex::new(1),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
            )
            .route("/users", web::get().to(get_users))
            .route("/users/{id}", web::get().to(get_user_by_id))
            .route("/users", web::delete().to(delete_all_users))
            .route("/users", web::post().to(add_user))
            .route("/users/{id}", web::put().to(update_user_by_id))
            .route("/users/{id}", web::patch().to(update_user_hours))
            .route("/users/{id}", web::delete().to(delete_user_by_id))
    })
    .bind("127.0.0.1:5003")?
    .run()
    .await
}
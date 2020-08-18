#[macro_use]
extern crate lazy_static;
extern crate captcha;
extern crate base64;
extern crate actix_web;
extern crate serde_json;
extern crate serde;
extern crate env_logger;
extern crate uuid;
extern crate chrono;

use captcha::{Captcha, Geometry};
use captcha::filters::{Cow, Noise, Wave};
use base64::{encode, decode};
use uuid::Uuid;
use chrono::prelude::*;

use std::path::Path;
use std::collections::HashMap;
use std::sync::Mutex;
use std::{thread, time};

use serde::{Serialize, Deserialize, Serializer};

use actix_web::{
    web,
    App,
    HttpRequest,
    HttpResponse,
    Responder,
    HttpServer,
    Result,

    middleware::Logger,
};

use env_logger::Env;
use std::borrow::{Borrow, BorrowMut};
use std::ops::Add;


lazy_static! {
    static ref HASHMAP: Mutex<HashMap<String, ChaValue>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

const MAX_COUNTER: i32 = 1000000;

#[derive(Serialize)]
struct Img {
    content: String,
    uuid_value: String,
}

#[derive(Serialize)]
struct ChaValue {
    value: String,
    exp_at: chrono::DateTime<chrono::Local>,
}

impl Img {
    fn new(_content: String, _uuid: String) -> Img {
        Img { content: _content, uuid_value: _uuid }
    }
}

async fn cha(_req: HttpRequest) -> impl Responder {
    let mut pic = Captcha::new();
    pic.add_chars(5)
        .apply_filter(Noise::new(0.3))
        .apply_filter(Wave::new(1.0, 10.0))
        .view(160, 60)
        .apply_filter(Cow::new().min_radius(20).max_radius(20).circles(1).area(Geometry::new(10, 100, 30, 30)));
    println!("{:?}", pic.chars());
    match pic.as_tuple() {
        Some(c) => {
            HttpResponse::Ok().header("content-type", "image.toml").body(c.1)
        },
        None => {
            HttpResponse::InternalServerError().body("failed")
        }
    }
}

async fn chapta(_req: HttpRequest) -> impl Responder {
    let mut pic = Captcha::new();
    pic.add_chars(5)
        .apply_filter(Noise::new(0.3))
        .apply_filter(Wave::new(1.0, 10.0))
        .view(160, 60)
        .apply_filter(Cow::new().min_radius(20).max_radius(20).circles(1).area(Geometry::new(10, 100, 30, 30)));
    match pic.as_tuple() {
        Some(c) => {
            let _uuid = Uuid::new_v4().to_string();
            let mut map = HASHMAP.lock().unwrap();
            if map.len() >= MAX_COUNTER as usize {
                format!("{:?}", "cha counter bigger than 10^7, retry later!!!");
            }
            map.insert((&*_uuid).parse().unwrap(), ChaValue { value: c.0, exp_at: Local::now().add(chrono::Duration::minutes(3)) });
            HttpResponse::Ok().json(Img::new(String::from("data:image.toml/png;base64,") + &*encode(c.1), _uuid.to_string()))
        },
        None => {
            HttpResponse::InternalServerError().body("failed")
        }
    }
}

#[derive(Deserialize)]
struct ChaVerify {
    uuid_value: String,
    cha_value: String,
}

async fn verify(verify_value: web::Json<ChaVerify>) -> impl Responder {
    let mut map = HASHMAP.lock().unwrap();
    let result = map.get(&*verify_value.uuid_value);
    match result {
        Some(v) => {
            let mut b  = HashMap::new();
            if v.value.to_uppercase() == verify_value.cha_value.to_uppercase() && chrono::Local::now().lt(&v.exp_at) {
                b.insert("verify", true);
                map.remove(&*verify_value.uuid_value);
                HttpResponse::Ok().json(b)
            } else {
                b.insert("verify", false);
                map.remove(&*verify_value.uuid_value);
                HttpResponse::Ok().json(b)
            }
        }
        None => {
            let mut b  = HashMap::new();
            b.insert("verify", false);
            map.remove(&*verify_value.uuid_value);
            HttpResponse::Ok().json(b)
        }
    }
}

async fn view(_req: HttpRequest) -> impl Responder {
    let map = HASHMAP.lock().unwrap();
    let mut con = vec![];
    for item in map.iter() {
        con.push(item)
    }
    HttpResponse::Ok().json(con)
}

struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

async fn _index(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard
    format!("Request number: {}", counter) // <- response with count
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::from_env(Env::default().default_filter_or("debug")).init();
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    thread::spawn(move || {
        loop {
            thread::sleep(time::Duration::from_secs(30));
            let mut map = HASHMAP.lock().unwrap();
            map.retain(|_key, value| {
                Local::now().lt(&value.exp_at)
            });
            println!("capacity:{:?}", map.len());
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .wrap(Logger::default())
            // .wrap(Logger::new("%a %{User-Agent}i"))
            .service(
                web::scope("/app")
                    .route("cha", web::get().to(chapta))
                    .route("img", web::get().to(cha))
                    .route("verify", web::post().to(verify))
                    // .route("index", web::get().to(_index))
                    // .route("view", web::get().to(view)) //only for debug
            )
    })
        .bind("0.0.0.0:8088")?
        .run()
        .await
}
#[macro_use]
extern crate diesel;

use favourite_streams::{get_favourite_streams, post_favourite_stream};
use rocket::{launch, routes};
use rocket_contrib::database;
use rocket_contrib::databases::diesel::PgConnection;
use serde::Deserialize;
use stream_management::post_stream_management;

pub mod authenticate;
pub mod database;
mod favourite_streams;
pub mod schema;
pub mod service;
mod stream_management;

#[database("pg_conn")]

pub struct DbConn(PgConnection);

#[derive(Deserialize)]
pub struct GlobalConfig {
    auth_url: String,
}

#[launch]
fn rocket() -> rocket::Rocket {
    openssl_probe::init_ssl_cert_env_vars();
    env_logger::init();
    let rocket = rocket::ignite();
    let global_config: GlobalConfig = rocket.figment().extract().expect("global config");

    rocket
        .attach(DbConn::fairing())
        .manage(global_config)
        .mount(
            "/stream-config",
            routes![
                post_favourite_stream,
                get_favourite_streams,
                post_stream_management
            ],
        )
}

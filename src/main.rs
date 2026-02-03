#[macro_use]
extern crate rocket;

use rust_django_starter::build_rocket;

#[launch]
async fn rocket() -> _ {
    build_rocket().await
}
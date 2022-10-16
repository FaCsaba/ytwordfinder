use business_layer::search_for_word;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    serde::json::Json,
    Request, Response,
};

use crate::business_layer::VideoTimeSubtitle;

mod business_layer;
#[macro_use]
extern crate rocket;

/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PATCH, PUT, DELETE, HEAD, OPTIONS, GET",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[get("/api/getVideoLinks/<word>")]
fn get_link(word: String) -> Json<Vec<VideoTimeSubtitle>> {
    Json(search_for_word(word))
}

#[post("/api/downloadSubtitle/<link>")]
fn download_subtitle(link: String) {
    business_layer::download_subtitle(link, None).unwrap();
}

#[post("/api/downloadSubtitle/<link>/<lang>")]
fn download_subtitle_with_lang(link: String, lang: Option<String>) {
    business_layer::download_subtitle(link, lang).unwrap();
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Cors)
        .mount(
            "/",
            routes![get_link, download_subtitle, download_subtitle_with_lang],
        )
        .mount("/", routes![all_options])
}

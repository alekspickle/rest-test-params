//! Simple RESTFUL server to process different optional params.
//!
//! ## Expected input example 
//!
//! ```{"a":true,"b":true, "c": true, "d": 3.7 "e": 5, "f": 2, "case": "C1"}```
//!
//! *Any parameter is omittable, but due to the requirements it will result to incorrect request error.
//!
//! ## Task description:
//! RESTful API receiving parameters:
//! A: bool
//! B: bool
//! C: bool
//! D: float
//! E: int
//! F: int
//!
//! ## Expected output
//!
//! `{h: M|P|T, k: float}`
//!
//! The assignment consists of base expressions set and two custom set of
//! expressions that override / extend the base rules.
//!
//! Base
//!
//!     A && B && !C => H = M
//!     A && B && C => H = P
//!     !A && B && C => H = T
//!     [other] => [error]
//!
//!     H = M => K = D + (D * E / 10)
//!     H = P => K = D + (D * (E - F) / 25.5)
//!     H = T => K = D - (D * F / 30)
//!
//! Custom 1
//!
//!     H = P => K = 2 * D + (D * E / 100)
//!
//! Custom 2
//!
//!     A && B && !C => H = T
//!     A && !B && C => H = M
//!     H = M => K = F + D + (D * E / 100)
//!
//!
//! # Run:
//!
//! ``` RUST_LOG=info cargo run```
//!
//! # Test:
//!
//! ``` curl -H "Content-Type: application/json" -X POST -d '{"a":true,"b":true, "c": true, "d": 4.7, "e": 5, "f": 2, "case": "C1"}' localhost:3030/compute ```
//! 
//! ## Web framework of choice:
//! Actix has testing utilities included so it is a convenient choice.
//! (warp claims itself *right* web framework, but albeit nice trace it just too ubiquitous and unclear in terms of testing)
//!
//! ## Error handling
//! Error handling made with anyhow(parsing) + actix_error(web) crates.
//! 
//! ## Tests 
//! Tests feature main possibles scenarios, but not all combinations of params tested, of course.
//! Most incorrect scenarios will be processed in either
//!


use anyhow::{anyhow, Result};
use log::warn;

mod types;
use types::*;

use actix_web::{error, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};

async fn help() -> HttpResponse {
    HttpResponse::Ok().json(format!(
        "API expects several of these params. If you got the error, check task description. {:?}",
        Params::default()
    ))
}

///
async fn index() -> HttpResponse {
    HttpResponse::Ok().json("You are asking my help, doing so without parameters...")
}

/// This handler uses json extractor with limit
async fn compute_factory(
    data: web::Json<Params>,
    _req: HttpRequest,
) -> Result<HttpResponse, Error> {
    match compute(&data) {
        Ok(a) => Ok(HttpResponse::Ok().json(a)),
        Err(e) => {
            warn!("Could not compute value: {:?}", e);
            Err(error::ErrorBadRequest(format!("Wrong params: {:?}", data)))
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/compute").route(web::post().to(compute_factory)))
            .service(web::resource("/help").route(web::get().to(help)))
    })
    .bind("127.0.0.1:3030")?
    .run()
    .await
}

fn compute(p: &Params) -> Result<Output> {
    let Params { a, b, c, .. } = p;
    let case = p.case.clone().map_or(Case::B, |v| v);

    match case {
        Case::B | Case::C1 => match (a, b, c) {
            (Some(true), Some(true), Some(false)) => output(H::M, &p, case),
            (Some(true), Some(true), Some(true)) => output(H::P, &p, case),
            (Some(false), Some(true), Some(true)) => output(H::T, &p, case),
            (_, _, _) => output(H::E, &p, case),
        },
        Case::C2 => match (a, b, c) {
            (Some(true), Some(true), Some(false)) => output(H::M, &p, case),
            (Some(true), Some(false), Some(true)) => output(H::M, &p, case),
            (Some(true), Some(true), Some(true)) => output(H::P, &p, case),
            (Some(false), Some(true), Some(true)) => output(H::T, &p, case),
            (_, _, _) => output(H::E, &p, case),
        },
    }
}

fn output(h: H, p: &Params, case: Case) -> Result<Output> {
    // TODO: figure out how to convert D, F, E params from Option<T> to T
    // and pass error if it rises in essential places (basically every expect(..))
    let d = p.d.expect("no D param");

    match h {
        H::M => {
            let e: f64 = p.e.expect("no E param").into();

            let k = match case {
                Case::C2 => {
                    let f: f64 = p.f.expect("no F param").into();
                    f + d + ((d * e) / 100.0)
                }
                _ => d + (d * e / 10.0),
            };

            Ok(Output { h: H::M, k })
        }
        H::P => {
            let e: f64 = p.e.expect("no E param").into();
            let f: f64 = p.f.expect("no F param").into();

            let k = match case {
                Case::C1 => 2.0 * d + ((d * e) / 100.0),
                _ => d + (d * (e - f) / 25.5),
            };

            Ok(Output { h: H::M, k })
        }
        H::T => {
            let f: f64 = p.f.expect("no F param").into();

            Ok(Output {
                h: H::M,
                k: d - (d * f / 30.0),
            })
        }
        H::E => Err(anyhow!("Set of parameters is not supported.")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::Service;
    use actix_web::{http, test, web, App};

    #[actix_rt::test]
    async fn correct_input() -> Result<(), Error> {
        let mut app = test::init_service(
            App::new().service(web::resource("/compute").route(web::post().to(compute_factory))),
        )
        .await;

        // {"a":true,"b":true, "c": true, "d": 3.7 "e": 5, "f": 2, "case": "C1"}
        let req = test::TestRequest::post()
            .uri("/compute")
            .set_json(&Params {
                a: Some(true),
                b: Some(true),
                c: Some(true),
                d: Some(3.7),
                e: Some(5),
                f: Some(2),
                case: Some(Case::C1),
            })
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };

        assert_eq!(response_body, r##"{"h":"M","k":7.585}"##);

        Ok(())
    }

    #[actix_rt::test]
    async fn incorrect_base_input() -> Result<(), Error> {
        let mut app = test::init_service(
            App::new().service(web::resource("/compute").route(web::post().to(compute_factory))),
        )
        .await;

        // {"a":false, "b":false, "c": false, "d": 3.7 "e": 5, "f": 2}
        let req = test::TestRequest::post()
            .uri("/compute")
            .set_json(&Params {
                a: Some(false),
                b: Some(false),
                c: Some(false),
                d: Some(3.7),
                e: Some(5),
                f: Some(2),
                case: None,
            })
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };

        let body = std::str::from_utf8(&response_body[0..12]).unwrap();
        assert_eq!(body, r#"Wrong params"#);

        Ok(())
    }

    #[actix_rt::test]
    async fn correct_c1_input() -> Result<(), Error> {
        let mut app = test::init_service(
            App::new().service(web::resource("/compute").route(web::post().to(compute_factory))),
        )
        .await;

        // {"a":false, "b":false, "c": false, "d": 3.7 "e": 5, "f": 2}
        let req = test::TestRequest::post()
            .uri("/compute")
            .set_json(&Params {
                a: Some(false),
                b: Some(true),
                c: Some(true),
                d: Some(3.7),
                e: Some(5),
                f: Some(2),
                case: Some(Case::C1),
            })
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };

        assert_eq!(response_body, r#"{"h":"M","k":3.4533333333333336}"#);

        Ok(())
    }
    #[actix_rt::test]
    async fn incorrect_c1_input() -> Result<(), Error> {
        let mut app = test::init_service(
            App::new().service(web::resource("/compute").route(web::post().to(compute_factory))),
        )
        .await;

        // {"a":false, "b":false, "c": false, "d": 3.7 "e": 5, "f": 2}
        let req = test::TestRequest::post()
            .uri("/compute")
            .set_json(&Params {
                a: Some(true),
                b: Some(false),
                c: Some(true),
                d: Some(3.7),
                e: Some(5),
                f: Some(2),
                case: Some(Case::C1),
            })
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };
        let body = std::str::from_utf8(&response_body[0..12]).unwrap();

        assert_eq!(body, r#"Wrong params"#);

        Ok(())
    }
    #[actix_rt::test]
    async fn correct_c2_input() -> Result<(), Error> {
        let mut app = test::init_service(
            App::new().service(web::resource("/compute").route(web::post().to(compute_factory))),
        )
        .await;

        // {"a":false, "b":false, "c": false, "d": 3.7 "e": 5, "f": 2}
        let req = test::TestRequest::post()
            .uri("/compute")
            .set_json(&Params {
                a: Some(true),
                b: Some(false),
                c: Some(true),
                d: Some(3.7),
                e: Some(5),
                f: Some(2),
                case: Some(Case::C2),
            })
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };

        assert_eq!(response_body, r#"{"h":"M","k":5.885}"#);

        Ok(())
    }
}

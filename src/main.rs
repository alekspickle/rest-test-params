use std::convert::Infallible;
use warp::{get, http::StatusCode, path, post, Filter, Rejection, Reply};
use anyhow::{anyhow, Result};
use log::info;

#[cfg(test)]
mod tests;

mod types;
use types::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // POST /compute  {"a": "true", "b": "false"}
    let compute = post()
        .and(path("compute"))
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|params: Params| {
            info!("params {:?}", params);
            let output = compute(&params).expect("Could not compute value");

            if let H::E = output.h {
                let code = StatusCode::BAD_REQUEST;
                warp::reply::json(&ErrorMessage {
                    message: "Invalid params format\n".into(),
                    code: code.as_u16(),
                })
            } else {
                warp::reply::json(&output)
            }
        });

    // GET /help
    let help = get().and(path("help")).map(|| format!("API expects several of these params. If you got the error, check task description. {:?}", Params::default()));

    let routes = help.or(compute).recover(handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await
}

fn compute(p: &Params) -> Result<Output> {
    let Params { a, b, c, .. } = p;

    match (a, b, c) {
        (Some(true), Some(true), Some(false)) => output(H::M, &p),
        (Some(true), Some(true), Some(true)) => output(H::P, &p),
        (Some(false), Some(true), Some(true)) => output(H::T, &p),
        (_, _, _) => output(H::E, &p),
    }
}

fn output(h: H, p: &Params) -> Result<Output> {
    if p.d.is_none() {
        return Err(anyhow!("no D param"))
    }
    
    let d = p.d.expect("no D param");

    match h {
        H::M => {
            let e: f64 = p.e.expect("no E param").into();

            Ok(Output {
                h: H::M,
                k: d + (d * e / 10.0),
            })
        }
        H::P => {
            let e: f64 = p.e.expect("no E param").into();
            let f: f64 = p.f.expect("no F param").into();

            Ok(Output {
                h: H::M,
                k: d + (d * (e - f) / 25.5),
            })
        }
        H::T => {
            let f: f64 = p.f.expect("no F param").into();

            Ok(Output {
                h: H::M,
                k: d - (d * f / 30.0),
            })
        }
        H::E => Ok(Output { h: H::E, k: 0.0 }),
    }
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(InvalidFormat) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "INVALID_PARAMS_FORMAT";
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}
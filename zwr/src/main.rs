#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use std::str::FromStr;

use warp::{Filter, fs, header, host, path, redirect, reject, reply};

#[derive(Debug)]
struct NoHost;

impl reject::Reject for NoHost {}

const NO_HOST_MESSAGE: &str = "Request is missing required `Host` header";

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            "info",
        )
    }
    pretty_env_logger::init();

    let ui_static = path("ui")
        .and(fs::dir("./ui"));

    let favicon = path("favicon.ico")
        .and(path::end())
        .and(fs::dir("./ui"));

    let robots = path("robots.txt")
        .and(path::end())
        .and(fs::dir("./ui"));

    let ui_home = path!("ui" / "home")
        .and(fs::file("./ui/index.html"));

    let ui_login = path!("ui" / "login")
        .and(fs::file("./ui/index.html"));

    let ui_explore = path!("ui" / "explore")
        .and(fs::file("./ui/index.html"));

    let ui_image = path!("ui" / "image" / ..)
        .and(fs::file("./ui/index.html"));

    let redirect = warp::any()
        .and(path::full()
            .and_then(|full_path: path::FullPath| async move {
                let parts: Vec<String> = full_path
                    .as_str()
                    .split(':')
                    .map(|s| s.to_string())
                    .collect();

                match parts.iter().any(|e| e.starts_with("/ui")) {
                    true => Err(reject::not_found()),
                    false => Ok(parts)
                }
            }))
        .and(host::optional()
            .and(header::optional::<String>("x-forwarded-host"))
            .and_then(|auth: Option<host::Authority>, forwarded_host: Option<String>| async move {
                if let Some(f) = forwarded_host {
                    if let Ok(a) = host::Authority::from_str(f.as_str()) { return Ok(a); }
                }

                if let Some(a) = auth {
                    Ok(a)
                } else {
                    Err(reject::custom(NoHost))
                }
            }))
        .and(header::optional::<String>("x-forwarded-proto"))
        .map(|parts: Vec<String>,
              authority: host::Authority,
              forwarded_proto: Option<String>,
        | {
            let path;
            match parts.len() {
                1 => {
                    let repository = parts[0].strip_prefix('/')
                        .unwrap_or(parts[0].as_str());

                    if repository.is_empty() {
                        path = "/ui/".to_string()
                    } else {
                        path = format!("/ui/image/{}", urlencoding::encode(repository));
                    }
                }
                2 => {
                    let repository = parts[0].strip_prefix('/')
                        .unwrap_or(parts[0].as_str());

                    path = format!("/ui/image/{}/tag/{}", urlencoding::encode(repository), parts[1]);
                }
                _ => {
                    path = "/ui/".to_string()
                }
            }

            let proto;
            if let Some(p) = forwarded_proto {
                proto = match p.as_str() {
                    "https" => { "https" }
                    _ => { "http" }
                }
            } else {
                proto = "http"
            }
            let secure = matches!(proto, "https");

            let uri: http::Uri = http::Uri::builder()
                .scheme(proto)
                .authority(authority.clone())
                .path_and_query(path)
                .build()
                .unwrap();

            let cookie = cookie::Cookie::build("user", "anonymous")
                .domain(authority.host())
                .path("/")
                .secure(secure)
                .http_only(false)
                .max_age(cookie::time::Duration::seconds(30))
                .finish();

            let origin = parts.join(":");
            info!("Redirecting {} to {}",origin,uri);
            reply::with_header(
                redirect::found(uri),
                http::header::SET_COOKIE,
                cookie.to_string().as_str(),
            )
        });

    let routes = ui_home
        .or(favicon)
        .or(robots)
        .or(ui_login)
        .or(ui_explore)
        .or(ui_image)
        .or(ui_static)
        .or(redirect)
        .recover(|err: reject::Rejection| async move {
            match err.find() {
                Some(NoHost) => {

                    Ok(
                        reply::with_status(
                            reply::html(NO_HOST_MESSAGE),
                            http::StatusCode::BAD_REQUEST,
                        )
                    )
                }
                _ => Err(err)
            }
        });

    warp::serve(routes)
        .run(([0, 0, 0, 0], 8080))
        .await;
}


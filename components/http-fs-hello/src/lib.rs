use std::path::PathBuf;
use std::{fs, path::Path};

use wstd::http::{Body, Method, Request, Response, StatusCode};

const STATIC_ROOT_DIR: &str = "/assets";

#[wstd::http_server]
async fn main(req: Request<Body>) -> Result<Response<Body>, wstd::http::Error> {
    if req.method() != Method::GET && req.method() != Method::HEAD {
        let res = Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::empty())?;
        return Ok(res);
    }

    let mut request_path_str = req.uri().path().to_string();
    // Serve index.html for requests to the slash path ("/")
    if request_path_str.ends_with("/") {
        request_path_str += "index.html";
    }

    println!("Serving: {}", request_path_str);

    let file_path = PathBuf::from(STATIC_ROOT_DIR).join(request_path_str.trim_start_matches('/'));

    let response = match fs::read(&file_path) {
        Ok(contents) => {
            let mime_type = get_mime_type(&file_path);

            let body = if req.method() == Method::HEAD {
                Body::empty()
            } else {
                Body::from(contents)
            };

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime_type)
                .body(body)?
        }
        Err(e) => {
            println!("Error reading file {:?}: {}", file_path, e);
            let status = if e.kind() == std::io::ErrorKind::NotFound {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Response::builder().status(status).body(Body::empty())?
        }
    };

    Ok(response)
}

fn get_mime_type(path: &Path) -> &str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        // Default for unknown or binary files
        _ => "application/octet-stream",
    }
}

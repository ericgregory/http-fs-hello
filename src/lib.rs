use wstd::http::body::IncomingBody;
use wstd::http::server::{Finished, Responder};
use wstd::http::{IntoBody, Request, Response, StatusCode};
use wstd::io::empty;
use wasi::filesystem::preopens::get_directories;
use wasi::filesystem::types::{DescriptorFlags, OpenFlags, PathFlags};

mod wash_plugin;

#[wstd::http_server]
async fn main(req: Request<IncomingBody>, res: Responder) -> Finished {
    match req.uri().path_and_query().unwrap().as_str() {
        "/read-file" => read_file(req, res).await,
        "/" => home(req, res).await,
        _ => not_found(req, res).await,
    }
}

async fn home(_req: Request<IncomingBody>, res: Responder) -> Finished {
    res.respond(Response::new("Hello!\n".into_body())).await
}

async fn not_found(_req: Request<IncomingBody>, responder: Responder) -> Finished {
    let res = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(empty())
        .unwrap();
    responder.respond(res).await
}

async fn read_file(_req: Request<IncomingBody>, res: Responder) -> Finished {
    match read_text_file("sample.txt") {
        Ok(contents) => {
            res.respond(Response::new(contents.into_body())).await
        }
        Err(e) => {
            let error_msg = format!("Error reading file: {}\n", e);
            let response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error_msg.into_body())
                .unwrap();
            res.respond(response).await
        }
    }
}

fn read_text_file(filename: &str) -> Result<String, String> {
    // Get preopened directories
    let preopens = get_directories();

    if preopens.is_empty() {
        return Err("No preopened directories available. Volume may not be mounted correctly.".to_string());
    }

    // Get the first preopened directory
    let (dir_descriptor, _) = preopens
        .first()
        .ok_or_else(|| "Failed to get preopened directory".to_string())?;

    // Open the file for reading
    let path_flags = PathFlags::empty();
    let open_flags = OpenFlags::empty();
    let descriptor_flags = DescriptorFlags::READ;

    let file_descriptor = dir_descriptor
        .open_at(path_flags, filename, open_flags, descriptor_flags)
        .map_err(|e| format!("Failed to open file: {:?}", e))?;

    // Read file metadata to get size
    let stat = file_descriptor
        .stat()
        .map_err(|e| format!("Failed to get file stats: {:?}", e))?;

    let file_size = stat.size;

    // Read the file contents
    let (data, _) = file_descriptor
        .read(file_size, 0)
        .map_err(|e| format!("Failed to read file: {:?}", e))?;

    // Convert bytes to string
    String::from_utf8(data)
        .map_err(|e| format!("File contains invalid UTF-8: {}", e))
}

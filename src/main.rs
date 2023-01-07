use std::io::{stdout, Write};
use tiny_http::{Request, Response, Server};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let bind = match &args[..] {
        [_program] => "127.0.0.1:8000",
        [_program, bind] if bind != "--help" => bind,
        [program, ..] => {
            print_help(&program);
            return Ok(());
        }
        [] => unreachable!(),
    };

    let server = Server::http(bind).unwrap();
    println!("Listening at {}\n", server.server_addr().to_string());

    for request in server.incoming_requests() {
        if let Err(err) = handle_request(request) {
            eprintln!("{err:?}");
        }
    }

    Ok(())
}

fn print_help(program: &str) {
    let program = program.split("/").last().unwrap_or("httprint");
    println!("Usage: {program} [ADDRESS]\n");
    println!("Parameters:\n  ADDRESS\tAddress to listen (default: 127.0.0.1:8000)")
}

fn handle_request(mut request: Request) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout().lock();

    let request_line = get_request_line(&request);
    writeln!(stdout, "{request_line}")?;

    let headers = get_headers(&request);
    writeln!(stdout, "{headers}")?;

    if let Some(body) = get_body(&mut request)? {
        writeln!(stdout, "\n{body}")?;
    }

    writeln!(stdout, "\n---\n")?;

    let response = Response::new_empty(200.into());
    request.respond(response)?;

    Ok(())
}

fn get_request_line(request: &Request) -> String {
    let method = request.method();
    let url = request.url();
    let http_version = request.http_version();

    format!("{method} {url} HTTP/{http_version}")
}

fn get_headers(request: &Request) -> String {
    request
        .headers()
        .into_iter()
        .map(|header| {
            let field = header.field.to_string();
            let value = header.value.to_string();
            format!("{field}: {value}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn get_body(request: &mut Request) -> Result<Option<String>, std::io::Error> {
    let body_length = request.body_length().unwrap_or(0);

    if body_length == 0 {
        return Ok(None);
    }

    let content_type: String = request
        .headers()
        .iter()
        .filter_map(|h| {
            if h.field.to_string().to_lowercase() == "content-type" {
                Some(h.value.to_string())
            } else {
                None
            }
        })
        .next()
        .unwrap_or("text/plain".to_owned());

    if content_type.contains("application/octet-stream") {
        return Ok(Some("[binary data]".to_owned()));
    }

    let body = {
        let reader = request.as_reader();
        let mut buf = Vec::with_capacity(body_length);
        reader.read_to_end(&mut buf)?;

        match String::from_utf8(buf) {
            Ok(data) => data,
            Err(_) => "[non utf8 data]".to_owned(),
        }
    };

    Ok(Some(body))
}

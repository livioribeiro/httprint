use tiny_http::{Request, Response, Server};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let bind = match &args[..] {
        [_program] => "127.0.0.1:8000",
        [_program, bind] if bind != "--help" => bind,
        [program, ..] => {
            print_usage(program);
            return Ok(());
        }
        [] => unreachable!(),
    };

    let server = Server::http(bind).unwrap();
    println!("Listening at {}\n", server.server_addr());

    for request in server.incoming_requests() {
        if let Err(err) = handle_request(request) {
            eprintln!("{err:?}");
        }
    }

    Ok(())
}

fn print_usage(program: &str) {
    let program = program.split('/').last().unwrap_or("httprint");
    println!("Usage: {program} [ADDRESS]\n");
    println!("Parameters:\n  ADDRESS\tAddress to listen (default: 127.0.0.1:8000)")
}

fn handle_request(mut request: Request) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = String::new();
    buf.push_str(get_request_line(&request).as_ref());

    let output = {
        let request_line = get_request_line(&request);
        let headers = get_headers(&request);

        if let Some(body) = get_body(&mut request)? {
            format!("{request_line}\n{headers}\n\n{body}\n\n---\n")
        } else {
            format!("{request_line}\n{headers}\n\n---\n")
        }
    };

    println!("{output}");

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
        .iter()
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

    let content_type = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Content-Type"))
        .map_or("text/plain".to_owned(), |h| h.value.to_string());

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

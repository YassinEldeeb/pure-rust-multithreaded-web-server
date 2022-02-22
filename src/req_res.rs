use std::fs;

use indexmap::IndexMap;

/// `Response` is a struct that takes the `buffer` from `TcpStream`
/// and can call methods on it to respond to the requester through different ways.
pub struct Response<'a> {
    buffer: &'a [u8],
}

/// `Request` is a struct that takes the `buffer` from `TcpStream`
/// and can call methods on it to parse the request buffer to a `ParsedRequest`
/// which have all of the useful information you would like to deal with.
pub struct Request<'a> {
    buffer: &'a [u8],
}

/// `ParsedRequest` is a return type of the `parse` method on the `Request` struct
/// It contains all of the metadata extracted from request data buffer.
#[derive(Debug, PartialEq)]
pub struct ParsedRequest {
    method: String,
    uri: String,
    http_version: f64,
    headers: IndexMap<String, String>,
    body: String,
}

impl<'a> Request<'a> {
    /// Instantiate a new `Request` struct by providing a request buffer data as the only argument.
    pub fn new(buffer: &'a [u8]) -> Request<'a> {
        Request { buffer }
    }

    /// Parses the request buffer and returns a `Option<ParsedRequest>`
    /// In the `Some<ParsedRequest>` case means that it was able to parse the buffer successfully.
    /// In the `None` case it means that the buffer was malformed and it could't able to parse it.
    ///
    /// You can find all kind of useful data from the parsed request buffer like:
    /// - method
    /// - uri
    /// - http version
    /// - headers
    /// - body
    pub fn parse(&self) -> Option<ParsedRequest> {
        let req_str = String::from_utf8_lossy(self.buffer);

        let mut headers = IndexMap::new();
        let mut body = String::from("");

        let mut lines: Vec<&str> = req_str.lines().collect();

        let mut parts = lines[0].split(' ');

        let method = parts.next()?.to_string();
        let uri = parts.next()?.to_string();
        let http_version = parts
            .next()?
            .replace("HTTP/", "")
            .parse()
            .expect("Couldn't parse http version!");

        lines.remove(0);

        for (idx, &i) in lines.iter().enumerate() {
            if i.is_empty() {
                body = lines[idx + 1].trim().replace("\u{0}", "");
            }

            let pair: Vec<&str> = i.split(':').map(|e| e.trim()).collect();
            if pair.len() >= 2 {
                headers.insert(pair[0].to_string(), pair[1].to_string());
            }
        }

        Some(ParsedRequest {
            body,
            headers,
            http_version,
            method,
            uri,
        })
    }
}

impl<'a> Response<'a> {
    /// Instantiate a new `Response` struct by providing a request buffer data as the only argument.
    pub fn new(buffer: &'a [u8]) -> Response<'a> {
        Response { buffer }
    }

    /// Instantiate a new `Request` struct from the provided request buffer data.
    /// Uses the `parse` method on the `Request` struct to get metadata about the request
    /// Decide based on the `uri` field in the `ParsedRequest` struct which page to display.
    ///
    ///
    /// If the parsing process failed, It'll return a response string with a 400 status code to indicate
    /// that the request was malformed.
    ///
    /// Then it reads the corresponding HTML file from the file system and returns back a well-formatted
    /// response string with the status code and the page contents.
    pub fn get_page(&self) -> String {
        let req = match Request::new(self.buffer).parse() {
            Some(v) => v,
            None => return response(400, "Bad ass Request", "", ""),
        };

        let page_path = if req.uri == "/" {
            String::from("frontend/index.html")
        } else {
            format!(
                "frontend{}{}",
                req.uri,
                if req.uri.contains(".html") {
                    ""
                } else {
                    ".html"
                }
            )
        };

        let content = fs::read_to_string(page_path)
            .unwrap_or_else(|_| fs::read_to_string("frontend/404.html").unwrap());

        response(
            200,
            "OK",
            &format!("Content-Length: {}", content.len()),
            &content,
        )
    }
}

/// A useful helper function for formatting the response string for easy re-use
/// It constructs a well-formatted response string using the provided arguments
/// `status`, `desc`, `headers` and the `body` of the response.
pub fn response(status: i32, desc: &str, headers: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {} {}\r\n{}\r\n\r\n{}",
        status, desc, headers, body
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta;

    const SAMPLE_REQ_STR: &str = "GET / HTTP/1.1
Host: 127.0.0.1:3000
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:97.0) Gecko/20100101 Firefox/97.0
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8
Accept-Language: en-US,en;q=0.5
Accept-Encoding: gzip, deflate
Connection: keep-alive
Upgrade-Insecure-Requests: 1
Sec-Fetch-Dest: document
Sec-Fetch-Mode: navigate
Sec-Fetch-Site: none
Sec-Fetch-User: ?1
Cache-Control: max-age=0";

    #[test]
    fn it_parses_request_buffer() {
        let req = Request::new(SAMPLE_REQ_STR.as_bytes());

        insta::assert_debug_snapshot!(req.parse().unwrap());
    }

    #[test]
    fn it_responds_correctly() {
        let res = Response::new(SAMPLE_REQ_STR.as_bytes());

        res.get_page().starts_with("HTTP/1.1 200 OK\r\n");
    }

    #[test]
    fn it_refuses_invalid_requests() {
        let req_str = SAMPLE_REQ_STR.replace("/ HTTP/1.1", "/not-found HTTP/1.1");
        let res = Response::new(req_str.as_bytes());

        res.get_page()
            .starts_with("HTTP/1.1 400 Bad ass Request\r\n");
    }

    #[test]
    fn it_formats_response_correctly() {
        assert_eq!(
            response(
                200,
                "OK",
                "Content-Type: application/json",
                "{ logged_in: true }"
            ),
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{ logged_in: true }"
        );

        assert_eq!(
            response(400, "Bad ass", "", ""),
            "HTTP/1.1 400 Bad ass\r\n\r\n\r\n"
        );
    }
}

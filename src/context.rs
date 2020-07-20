use bytes::Bytes;
use http::request::Parts;
use hyper::{Body, Error, Response, StatusCode};
use std::collections::HashMap;
use std::convert::TryInto;
use std::str;
use thruster::context::hyper_request::HyperRequest;
use thruster::middleware::query_params::HasQueryParams;
use thruster::Context;
use tokio::stream::StreamExt;

use crate::body::ProtoBody;
use crate::context::ProtoContext as Ctx;

pub fn generate_context(request: HyperRequest, _state: &(), _path: &str) -> Ctx {
    Ctx::new(request)
}

pub enum SameSite {
    #[allow(dead_code)]
    Strict,
    #[allow(dead_code)]
    Lax,
}

pub struct CookieOptions {
    pub domain: String,
    pub path: String,
    pub expires: u64,
    pub http_only: bool,
    pub max_age: u64,
    pub secure: bool,
    pub signed: bool,
    pub same_site: SameSite,
}

impl Default for CookieOptions {
    fn default() -> CookieOptions {
        CookieOptions {
            domain: "".to_owned(),
            path: "/".to_owned(),
            expires: 0,
            http_only: false,
            max_age: 0,
            secure: false,
            signed: false,
            same_site: SameSite::Strict,
        }
    }
}

#[derive(Default)]
pub struct ProtoContext {
    pub body: ProtoBody,
    pub query_params: HashMap<String, String>,
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub hyper_request: Option<HyperRequest>,
    http_version: hyper::Version,
    #[allow(dead_code)]
    request_body: Option<Body>,
    request_parts: Option<Parts>,
}

impl ProtoContext {
    pub fn new(req: HyperRequest) -> ProtoContext {
        let params = req.params.clone();
        let mut ctx = ProtoContext {
            body: ProtoBody::empty(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            status: 200,
            params,
            hyper_request: Some(req),
            request_body: None,
            request_parts: None,
            http_version: hyper::Version::HTTP_11,
        };

        ctx.set("Server", "Thruster");

        ctx
    }

    ///
    /// Set the body as a string
    ///
    pub fn body(&mut self, body_string: &str) {
        self.body = ProtoBody::from_bytes(Bytes::from(body_string.to_string()));
    }

    ///
    /// Get the body as a string
    ///
    #[allow(dead_code)]
    pub async fn get_body(self) -> Result<(String, ProtoContext), Error> {
        let ctx = match self.request_body {
            Some(_) => self,
            None => self.to_owned_request(),
        };

        let mut results = "".to_string();
        let mut unwrapped_body = ctx.request_body.unwrap();
        while let Some(chunk) = unwrapped_body.next().await {
            // TODO(trezm): Dollars to donuts this is pretty slow -- could make it faster with a
            // mutable byte buffer.
            results = format!("{}{}", results, String::from_utf8_lossy(chunk?.as_ref()));
        }

        Ok((
            results,
            ProtoContext {
                body: ctx.body,
                query_params: ctx.query_params,
                headers: ctx.headers,
                status: ctx.status,
                params: ctx.params,
                hyper_request: ctx.hyper_request,
                request_body: Some(Body::empty()),
                request_parts: ctx.request_parts,
                http_version: ctx.http_version,
            },
        ))
    }

    #[allow(dead_code)]
    pub fn parts(&self) -> &Parts {
        self.request_parts
            .as_ref()
            .expect("Must call `to_owned_request` prior to getting parts")
    }

    #[allow(dead_code)]
    pub fn to_owned_request(self) -> ProtoContext {
        match self.hyper_request {
            Some(hyper_request) => {
                let (parts, body) = hyper_request.request.into_parts();

                ProtoContext {
                    body: self.body,
                    query_params: self.query_params,
                    headers: self.headers,
                    status: self.status,
                    params: hyper_request.params,
                    hyper_request: None,
                    request_body: Some(body),
                    request_parts: Some(parts),
                    http_version: self.http_version,
                }
            }
            None => self,
        }
    }

    ///
    /// Set the response status code
    ///
    #[allow(dead_code)]
    pub fn status(&mut self, code: u32) {
        self.status = code.try_into().unwrap();
    }

    ///
    /// Set the response `Content-Type`. A shortcode for
    ///
    /// ```ignore
    /// ctx.set("Content-Type", "some-val");
    /// ```
    ///
    #[allow(dead_code)]
    pub fn content_type(&mut self, c_type: &str) {
        self.set("Content-Type", c_type);
    }

    ///
    /// Set up a redirect, will default to 302, but can be changed after
    /// the fact.
    ///
    /// ```ignore
    /// ctx.set("Location", "/some-path");
    /// ctx.status(302);
    /// ```
    ///
    #[allow(dead_code)]
    pub fn redirect(&mut self, destination: &str) {
        self.status(302);

        self.set("Location", destination);
    }

    ///
    /// Sets a cookie on the response
    ///
    #[allow(dead_code)]
    pub fn cookie(&mut self, name: &str, value: &str, options: &CookieOptions) {
        let cookie_value = match self.headers.get("Set-Cookie") {
            Some(val) => format!("{}, {}", val, self.cookify_options(name, value, &options)),
            None => self.cookify_options(name, value, &options),
        };

        self.set("Set-Cookie", &cookie_value);
    }

    #[allow(dead_code)]
    fn cookify_options(&self, name: &str, value: &str, options: &CookieOptions) -> String {
        let mut pieces = vec![format!("Path={}", options.path)];

        if options.expires > 0 {
            pieces.push(format!("Expires={}", options.expires));
        }

        if options.max_age > 0 {
            pieces.push(format!("Max-Age={}", options.max_age));
        }

        if !options.domain.is_empty() {
            pieces.push(format!("Domain={}", options.domain));
        }

        if options.secure {
            pieces.push("Secure".to_owned());
        }

        if options.http_only {
            pieces.push("HttpOnly".to_owned());
        }

        match options.same_site {
            SameSite::Strict => pieces.push("SameSite=Strict".to_owned()),
            SameSite::Lax => pieces.push("SameSite=Lax".to_owned()),
        };

        format!("{}={}; {}", name, value, pieces.join(", "))
    }

    #[allow(dead_code)]
    pub fn set_http2(&mut self) {
        self.http_version = hyper::Version::HTTP_2;
    }

    #[allow(dead_code)]
    pub fn set_http11(&mut self) {
        self.http_version = hyper::Version::HTTP_11;
    }

    #[allow(dead_code)]
    pub fn set_http10(&mut self) {
        self.http_version = hyper::Version::HTTP_10;
    }
}

impl Context for ProtoContext {
    type Response = Response<ProtoBody>;

    fn get_response(mut self) -> Self::Response {
        let mut response_builder = Response::builder();

        for (key, val) in self.headers {
            let key: &str = &key;
            let val: &str = &val;
            response_builder = response_builder.header(key, val);
        }

        // if self.status < 200 || self.status > 399 {
        //     // self.body.
        // }
        if self.body.proto_status() != 0 {
            self.body.set_body(Body::empty());
        }

        response_builder
            .status(StatusCode::from_u16(200).unwrap())
            .version(self.http_version)
            .body(self.body)
            .unwrap()
    }

    fn set_body(&mut self, body: Vec<u8>) {
        self.body = ProtoBody::from_bytes(Bytes::from(body));
    }

    fn set_body_bytes(&mut self, bytes: Bytes) {
        self.body = ProtoBody::from_bytes(bytes);
    }

    fn route(&self) -> &str {
        let uri = self.hyper_request.as_ref().unwrap().request.uri();

        match uri.path_and_query() {
            Some(val) => val.as_str(),
            None => uri.path(),
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_owned(), value.to_owned());
    }

    fn remove(&mut self, key: &str) {
        self.headers.remove(key);
    }
}

impl HasQueryParams for ProtoContext {
    fn set_query_params(&mut self, query_params: HashMap<String, String>) {
        self.query_params = query_params;
    }
}

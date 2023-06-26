use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use hyper::body::Bytes;
use hyper::header::{HeaderName, HeaderValue};
use hyper::{HeaderMap, Response, StatusCode};
use prost::{DecodeError, Message};
use std::collections::HashMap;
use std::convert::TryInto;
use std::str;
use thruster::context::hyper_request::HyperRequest;
use thruster::middleware::query_params::HasQueryParams;
use thruster::Context;
use tokio_stream::StreamExt;

use crate::body::ProtoBody;
use crate::context::ProtoContext as Ctx;

/// 4 accounts for content-type, server, grpc-status, and trailers.
const DEFAULT_HEADER_CAPACITY: usize = 4;

pub fn generate_context(request: HyperRequest, _state: &(), _path: &str) -> Ctx<()> {
    Ctx::new(request, ())
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
pub struct ProtoContext<T> {
    pub body: Option<ProtoBody>,
    pub query_params: Option<HashMap<String, String>>,
    pub status: u16,
    pub hyper_request: Option<HyperRequest>,
    pub extra: T,
    http_version: hyper::Version,
    headers: HeaderMap,
}

impl<T> ProtoContext<T> {
    pub fn new(req: HyperRequest, extra: T) -> ProtoContext<T> {
        let mut ctx = ProtoContext {
            body: None,
            query_params: None,
            headers: HeaderMap::with_capacity(DEFAULT_HEADER_CAPACITY),
            status: 200,
            hyper_request: Some(req),
            extra,
            http_version: hyper::Version::HTTP_11,
        };

        ctx.set("Server", "Thruster");

        ctx
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
            Some(val) => format!(
                "{}, {}",
                val.to_str().unwrap_or_else(|_| ""),
                self.cookify_options(name, value, &options)
            ),
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

    pub fn set_proto_status(&mut self, status: u16) {
        self.headers
            .insert("grpc-status", format!("{}", status).parse().unwrap());
    }
}

impl<T> Context for ProtoContext<T> {
    type Response = Response<ProtoBody>;

    fn get_response(mut self) -> Self::Response {
        let mut body = self.body.take().unwrap();
        body.set_headers(self.headers.clone());
        let mut response = Response::new(body);

        *response.status_mut() = StatusCode::from_u16(self.status).unwrap();
        *response.headers_mut() = self.headers;
        *response.version_mut() = self.http_version;

        response
    }

    fn set_body(&mut self, body: Vec<u8>) {
        self.body.replace(ProtoBody::from_bytes(Bytes::from(body)));
    }

    fn set_body_bytes(&mut self, bytes: Bytes) {
        self.body.replace(ProtoBody::from_bytes(bytes));
    }

    fn route(&self) -> &str {
        let uri = self.hyper_request.as_ref().unwrap().request.uri();

        match uri.path_and_query() {
            Some(val) => val.as_str(),
            None => uri.path(),
        }
    }

    fn set(&mut self, key: &str, value: &str) {
        self.headers.insert(
            HeaderName::from_bytes(key.as_bytes()).unwrap(),
            HeaderValue::from_str(value).unwrap(),
        );
    }

    fn remove(&mut self, key: &str) {
        self.headers.remove(key);
    }
}

impl<T> HasQueryParams for ProtoContext<T> {
    fn set_query_params(&mut self, query_params: HashMap<String, String>) {
        self.query_params = Some(query_params);
    }
}

impl<T> Clone for ProtoContext<T> {
    fn clone(&self) -> Self {
        panic!("Do not use, just for internals.");
    }
}

#[async_trait]
trait ProtoContextExt<T> {
    async fn proto<M: Message + std::default::Default>(&mut self, message: M);
    async fn get_proto<M: Message + std::default::Default>(&mut self) -> Result<M, DecodeError>;
}

#[async_trait]
impl<T: Send> ProtoContextExt<T> for ProtoContext<T> {
    async fn proto<M: Message + std::default::Default>(&mut self, message: M) {
        self.set("content-type", "application/grpc");
        self.set("grpc-status", "0");
        self.set("trailers", "grpc-status");
        self.set_http2();

        let mut buf = BytesMut::new();
        buf.reserve(5);
        buf.put(&b"00000"[..]);

        let _ = message.encode(&mut buf);

        let len = buf.len() - 5;
        assert!(len <= std::u32::MAX as usize);
        {
            let mut buf = &mut buf[..5];
            buf.put_u8(0); // byte must be 0, reserve doesn't auto-zero
            buf.put_u32(len as u32);
        }
        let buf = buf.split_to(len + 5).freeze();

        self.body = Some(ProtoBody::from_bytes(buf));
    }

    async fn get_proto<M: Message + std::default::Default>(&mut self) -> Result<M, DecodeError> {
        let hyper_request = self.hyper_request.take().unwrap().request;

        let mut results = vec![];
        let mut body = hyper_request.into_body();
        while let Some(Ok(chunk)) = body.next().await {
            results.put(chunk);
        }

        M::decode(&results[5..])
    }
}

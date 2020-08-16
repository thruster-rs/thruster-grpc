use bytes::Bytes;
use core::task::{Context, Poll};
use http::header::HeaderMap;
use hyper::body::HttpBody;
use hyper::Body;
use std::pin::Pin;

fn create_default_header_map() -> HeaderMap {
    let mut header_map = HeaderMap::default();

    header_map.insert("grpc-status", "0".parse().unwrap());

    header_map
}

pub struct ProtoBody {
    inner: Body,
    header_map: Option<HeaderMap>,
}

impl ProtoBody {
    pub fn from_bytes(bytes: Bytes) -> ProtoBody {
        ProtoBody {
            inner: Body::from(bytes),
            header_map: None,
        }
    }

    pub fn empty() -> ProtoBody {
        ProtoBody {
            inner: Body::empty(),
            header_map: None,
        }
    }

    pub fn set_body(&mut self, body: Body) {
        self.inner = body;
    }

    pub fn set_headers(&mut self, headers: HeaderMap) {
        self.header_map = Some(headers);
    }
}

impl Default for ProtoBody {
    fn default() -> ProtoBody {
        ProtoBody {
            inner: Body::default(),
            header_map: Some(create_default_header_map()),
        }
    }
}

impl HttpBody for ProtoBody {
    type Data = Bytes;
    type Error = hyper::Error;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        Pin::new(&mut self.inner).poll_data(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(self.get_mut().header_map.take()))
    }
}

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
    header_map: HeaderMap,
    proto_status: u16,
}

impl ProtoBody {
    pub fn from_bytes(bytes: Bytes) -> ProtoBody {
        ProtoBody {
            inner: Body::from(bytes),
            header_map: create_default_header_map(),
            proto_status: 0,
        }
    }

    pub fn empty() -> ProtoBody {
        ProtoBody {
            inner: Body::empty(),
            header_map: create_default_header_map(),
            proto_status: 0,
        }
    }

    pub fn set_body(&mut self, body: Body) {
        self.inner = body;
    }

    pub fn set_proto_status(&mut self, status: u16) {
        self.proto_status = status;
        self.header_map
            .insert("grpc-status", format!("{}", status).parse().unwrap());
    }

    pub fn proto_status(&self) -> u16 {
        self.proto_status
    }
}

impl Default for ProtoBody {
    fn default() -> ProtoBody {
        ProtoBody {
            inner: Body::default(),
            header_map: create_default_header_map(),
            proto_status: 0,
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
        Poll::Ready(Ok(Some(self.header_map.clone())))
    }
}

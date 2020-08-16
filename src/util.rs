use bytes;
use bytes::buf::BufMut;
use bytes::BytesMut;
use prost::{DecodeError, Message};
use thruster::Context;
use tokio::stream::StreamExt;

use crate::body::ProtoBody;
use crate::context::ProtoContext as Ctx;

const DEFAULT_CAPACITY: usize = 256;

pub async fn context_to_message<T: Message + std::default::Default>(
    context: &mut Ctx,
) -> Result<T, DecodeError> {
    let mut hyper_request = context.hyper_request.take().unwrap().request;

    let mut results = BytesMut::with_capacity(DEFAULT_CAPACITY);
    while let Some(Ok(chunk)) = hyper_request.body_mut().next().await {
        results.reserve(chunk.len());
        results.put(chunk);
    }

    T::decode(&results[5..])
}

pub async fn message_to_context<T: Message + std::default::Default>(
    mut context: Ctx,
    message: T,
) -> Ctx {
    context.set("content-type", "application/grpc");
    context.set("grpc-status", "0");
    context.set("trailers", "grpc-status");
    context.set_http2();

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

    context.body.replace(ProtoBody::from_bytes(buf));

    context
}

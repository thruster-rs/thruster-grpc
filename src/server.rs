use std::net::ToSocketAddrs;

use async_trait::async_trait;
use hyper::service::{make_service_fn, service_fn, Service};
use hyper::{Body, Request, Response, Server};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};

use thruster::context::hyper_request::HyperRequest;
use thruster::{App, Context, ThrusterServer};

use crate::body::ProtoBody;

pub struct ProtoServer<T: 'static + Context + Send, S: Send> {
    app: App<HyperRequest, T, S>,
}

#[async_trait]
impl<T: Context<Response = Response<ProtoBody>> + Send, S: 'static + Send + Sync> ThrusterServer
    for ProtoServer<T, S>
{
    type Context = T;
    type Response = Response<ProtoBody>;
    type Request = HyperRequest;
    type State = S;

    fn new(app: App<Self::Request, T, Self::State>) -> Self {
        ProtoServer { app }
    }

    async fn build(mut self, host: &str, port: u16) {
        self.app._route_parser.optimize();

        let arc_app = Arc::new(self.app);
        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        async move {
            let service = make_service_fn(|_| {
                let app = arc_app.clone();

                futures::future::ready(Ok::<_, hyper::Error>(_ProtoService { app }))
            });

            let server = Server::bind(&addr)
                .tcp_keepalive(None)
                .tcp_nodelay(true)
                .http2_only(true)
                .http2_initial_connection_window_size(None)
                .http2_initial_stream_window_size(None)
                .http2_max_concurrent_streams(None)
                .serve(service);

            server.await?;

            Ok::<_, hyper::Error>(())
        }
        .await
        .expect("hyper server failed");
    }
}

struct _ProtoService<T: 'static + Context + Send, S: Send> {
    app: Arc<App<HyperRequest, T, S>>,
}

impl<T: 'static + Context + Send, S: 'static + Send> Service<Request<Body>>
    for _ProtoService<T, S>
{
    type Response = T::Response;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let matched = self.app.resolve_from_method_and_path(
            req.method().as_str(),
            req.uri().path_and_query().unwrap().as_str(),
        );

        let req = HyperRequest::new(req);
        Box::pin(self.app.resolve(req, matched))
    }
}

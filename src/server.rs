use std::net::ToSocketAddrs;

use async_trait::async_trait;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::sync::Arc;

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

                async {
                    Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                        let matched = app.resolve_from_method_and_path(
                            &req.method().to_string(),
                            &req.uri().path_and_query().unwrap().to_string(),
                        );

                        let req = HyperRequest::new(req);
                        let res = app.resolve(req, matched);

                        res
                    }))
                }
            });

            let server = Server::bind(&addr).serve(service);

            server.await?;

            Ok::<_, hyper::Error>(())
        }
        .await
        .expect("hyper server failed");
    }
}

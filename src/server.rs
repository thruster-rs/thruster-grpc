use std::net::ToSocketAddrs;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::sync::Arc;

use thruster::context::hyper_request::HyperRequest;
use thruster::{App, Context, ReusableBoxFuture, ThrusterServer};

use crate::body::ProtoBody;

pub struct ProtoServer<T: 'static + Context + Clone + Send + Sync, S: Send> {
    app: App<HyperRequest, T, S>,
}

impl<
        T: Context<Response = Response<ProtoBody>> + Clone + Send + Sync,
        S: 'static + Send + Sync,
    > ThrusterServer for ProtoServer<T, S>
{
    type Context = T;
    type Response = Response<ProtoBody>;
    type Request = HyperRequest;
    type State = S;

    fn new(app: App<Self::Request, T, Self::State>) -> Self {
        ProtoServer { app }
    }

    fn build(mut self, host: &str, port: u16) -> ReusableBoxFuture<()> {
        self.app = self.app.commit();

        let arc_app = Arc::new(self.app);
        let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();

        ReusableBoxFuture::new(async move {
            let service = make_service_fn(|socket: &hyper::server::conn::AddrStream| {
                let app = arc_app.clone();
                let ip = socket.remote_addr().ip();

                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                        let mut req = HyperRequest::new(req);
                        req.ip = Some(ip);

                        app.match_and_resolve(req)
                    }))
                }
            });

            let server = Server::bind(&addr).serve(service);

            server.await.expect("Hyper server failed to start");
        })
        // .await
        // .expect("hyper server failed");
    }
}

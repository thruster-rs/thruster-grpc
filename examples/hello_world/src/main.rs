use dotenv::dotenv;
use log::info;
use std::env;
use thruster::context::hyper_request::HyperRequest;
use thruster::errors::ThrusterError as Error;
use thruster::{async_middleware, map_try, middleware_fn};
use thruster::{App, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};
use thruster_grpc::context::{generate_context, ProtoContext as Ctx};
use thruster_grpc::error::ProtoErrorSet;
use thruster_grpc::server::ProtoServer;
use thruster_grpc::util::{context_to_message, message_to_context};

mod hello_world {
    include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));
}

#[middleware_fn]
pub async fn say_hello(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let hello_world_request = map_try!(context_to_message::<hello_world::HelloRequest>(&mut context).await, Err(_) => {
        Error::generic_error(context)
    });

    Ok(message_to_context(
        context,
        hello_world::HelloReply {
            message: format!("Hello, {}", hello_world_request.name),
        },
    )
    .await)
}

fn main() {
    let _ = dotenv();

    env_logger::init();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "4321".to_string());

    info!("Starting server at {}:{}", host, port);

    let mut app = App::<HyperRequest, Ctx, ()>::create(generate_context, ());
    app.post(
        "/helloworld.Greeter/SayHello",
        async_middleware!(Ctx, [say_hello]),
    );

    let server = ProtoServer::new(app);
    server.start(&host, port.parse::<u16>().unwrap());
}

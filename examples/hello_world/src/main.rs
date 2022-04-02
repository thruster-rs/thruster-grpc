use dotenv::dotenv;
use log::info;
use std::env;
use thruster::{
    context::hyper_request::HyperRequest, errors::ThrusterError as Error, m, map_try,
    middleware_fn, App, MiddlewareNext, MiddlewareResult, ThrusterServer,
};
use thruster_grpc::{
    context::{generate_context, ProtoContext as Ctx},
    error::ProtoErrorSet,
    server::ProtoServer,
    util::{context_to_message, message_to_context},
};

mod hello_world {
    include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));
}

#[middleware_fn]
pub async fn say_hello(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let hello_world_request = map_try!(context_to_message::<hello_world::HelloRequest>(&mut context).await, Err(_e) => {
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

#[middleware_fn]
pub async fn others(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let hello_world_request = map_try!(context_to_message::<hello_world::HelloRequest>(&mut context).await, Err(_e) => {
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

    let app = App::<HyperRequest, Ctx, ()>::create(generate_context, ())
        .post("/helloworld.Greeter/SayHello", m![say_hello])
        .post("/*", m![others]);

    let server = ProtoServer::new(app);
    server.start(&host, port.parse::<u16>().unwrap());
}

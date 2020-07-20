use thruster::errors::ThrusterError as Error;

use crate::context::ProtoContext as Ctx;
use crate::status::ProtoStatus;

pub trait ProtoErrorSet {
    fn already_exists_error(context: Ctx) -> Error<Ctx>;
    fn parsing_error(context: Ctx, error: &str) -> Error<Ctx>;
    fn generic_error(context: Ctx) -> Error<Ctx>;
    fn unauthorized_error(context: Ctx) -> Error<Ctx>;
    fn not_found_error(context: Ctx) -> Error<Ctx>;
}

impl ProtoErrorSet for Error<Ctx> {
    fn already_exists_error(mut context: Ctx) -> Error<Ctx> {
        context.status(409);
        context
            .body
            .set_proto_status(ProtoStatus::AlreadyExists as u16);
        Error {
            context,
            message: format!("Already exists"),
            status: 409,
            cause: None,
        }
    }

    fn parsing_error(mut context: Ctx, error: &str) -> Error<Ctx> {
        context.status(400);
        context
            .body
            .set_proto_status(ProtoStatus::InvalidArgument as u16);
        Error {
            context,
            message: format!("Failed to parse '{}'", error),
            status: 400,
            cause: None,
        }
    }

    fn generic_error(mut context: Ctx) -> Error<Ctx> {
        context.status(400);
        context
            .body
            .set_proto_status(ProtoStatus::InvalidArgument as u16);
        Error {
            context,
            message: "Something didn't work!".to_string(),
            status: 400,
            cause: None,
        }
    }

    fn unauthorized_error(mut context: Ctx) -> Error<Ctx> {
        context.status(401);
        context
            .body
            .set_proto_status(ProtoStatus::Unauthenticated as u16);
        Error {
            context,
            message: "Unauthorized".to_string(),
            status: 401,
            cause: None,
        }
    }

    fn not_found_error(mut context: Ctx) -> Error<Ctx> {
        context.status(404);
        context.body.set_proto_status(ProtoStatus::NotFound as u16);
        Error {
            context,
            message: "Not found".to_string(),
            status: 404,
            cause: None,
        }
    }
}

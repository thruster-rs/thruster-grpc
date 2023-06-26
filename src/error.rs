use thruster::errors::ThrusterError as Error;

use crate::context::ProtoContext as Ctx;
use crate::status::ProtoStatus;

pub trait ProtoErrorSet<T> {
    fn already_exists_error(context: Ctx<T>) -> Error<Ctx<T>>;
    fn parsing_error(context: Ctx<T>, error: &str) -> Error<Ctx<T>>;
    fn generic_error(context: Ctx<T>) -> Error<Ctx<T>>;
    fn unauthorized_error(context: Ctx<T>) -> Error<Ctx<T>>;
    fn not_found_error(context: Ctx<T>) -> Error<Ctx<T>>;
}

impl<T> ProtoErrorSet<T> for Error<Ctx<T>> {
    fn already_exists_error(mut context: Ctx<T>) -> Error<Ctx<T>> {
        context.status(409);
        context.set_proto_status(ProtoStatus::AlreadyExists as u16);
        Error {
            context,
            message: format!("Already exists"),
            cause: None,
        }
    }

    fn parsing_error(mut context: Ctx<T>, error: &str) -> Error<Ctx<T>> {
        context.status(400);
        context.set_proto_status(ProtoStatus::InvalidArgument as u16);
        Error {
            context,
            message: format!("Failed to parse '{}'", error),
            cause: None,
        }
    }

    fn generic_error(mut context: Ctx<T>) -> Error<Ctx<T>> {
        context.status(400);
        context.set_proto_status(ProtoStatus::InvalidArgument as u16);
        Error {
            context,
            message: "Something didn't work!".to_string(),
            cause: None,
        }
    }

    fn unauthorized_error(mut context: Ctx<T>) -> Error<Ctx<T>> {
        context.status(401);
        context.set_proto_status(ProtoStatus::Unauthenticated as u16);
        Error {
            context,
            message: "Unauthorized".to_string(),
            cause: None,
        }
    }

    fn not_found_error(mut context: Ctx<T>) -> Error<Ctx<T>> {
        context.status(404);
        context.set_proto_status(ProtoStatus::NotFound as u16);
        Error {
            context,
            message: "Not found".to_string(),
            cause: None,
        }
    }
}

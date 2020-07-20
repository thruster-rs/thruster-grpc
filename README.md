# Thruster-gRPC

This is an experimental library to provide gRPC support over [thruster](https://github.com/thruster-rs/thruster).

### Usage

*Note: Currently, this library only works with a hyper-based server of thruster.*

The brunt of the work is done by the `util::context_to_message` and `util::context_from_message` methods. These take proto inputs and make them into usable structs (and vice versa.) Also provided in this crate are a hyper-based server for handling incoming protos (in HTTP2,) and a dedicated context.

Please check the examples for usage, docs will be forthcoming soon, but are fairly sparse at the moment!

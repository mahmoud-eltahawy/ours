pub mod client;
pub mod error;
pub mod server;
pub mod top;
mod nav {
    use tonic::include_proto;

    include_proto!("nav.v1");
}

pub use nav::UnitKind;

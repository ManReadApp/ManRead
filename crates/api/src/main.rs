pub mod error;
mod init;
pub mod models;
mod routes;
pub mod services;
mod util;
pub use init::random_string;
pub use init::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init::init().await
}

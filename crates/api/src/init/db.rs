#[cfg(feature = "db")]
use std::sync::LazyLock;

#[cfg(feature = "db")]
use surrealdb::engine::remote::ws::Client;
#[cfg(feature = "db")]
use surrealdb::Surreal;
#[cfg(feature = "db")]
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root};

#[cfg(feature = "db")]
pub static DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init);
#[cfg(feature = "db")]
pub async fn init_db() -> Result<(), surrealdb::Error> {
    DB.connect::<Ws>("localhost:8083").await?;

    DB.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    DB.use_ns("manread").use_db("manread").await?;
    Ok(())
}

#[cfg(not(feature = "db"))]
pub async fn init_db() -> Result<(), ()> {
    Ok(())
}

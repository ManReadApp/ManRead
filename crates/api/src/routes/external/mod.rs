mod chapters;
mod info;
mod mangas;
mod search;
mod statistics;
mod uri;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/external")
        .service(search::register())
        .service(uri::register())
        .service(info::register())
        .service(mangas::register())
        .service(chapters::register())
        .service(statistics::register())
}

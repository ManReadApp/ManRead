mod cover;
pub mod cover_img;
mod page;
mod page_image;
mod page_translation;
mod upload;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/image")
        .service(cover::register())
        .service(page::register())
        .service(upload::register())
        .service(page_image::register())
        .service(page_translation::register())
}

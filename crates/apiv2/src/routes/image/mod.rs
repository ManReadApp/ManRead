mod cover;
pub mod cover_img;
mod page_image;
pub mod stream;
mod upload;

pub fn register() -> apistos::web::Scope {
    apistos::web::scope("/image")
        .service(cover::register())
        .service(upload::register())
        .service(page_image::register())
}

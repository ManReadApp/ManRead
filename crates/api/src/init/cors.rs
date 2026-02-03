use actix_cors::Cors;

pub fn init_cors() -> Cors {
    #[cfg(all(feature = "cors", not(feature = "cors-permissive")))]
    return actix_cors::Cors::default()
        .allow_any_header()
        .allowed_methods(vec!["GET", "POST"])
        .supports_credentials()
        .max_age(3600);
    #[cfg(all(feature = "cors", feature = "cors-permissive"))]
    return actix_cors::Cors::permissive();
    #[cfg(not(any(feature = "cors", feature = "cors-permissive")))]
    unreachable!("this function should only be called when cors is activated")
}

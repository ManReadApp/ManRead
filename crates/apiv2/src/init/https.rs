use std::path::Path;

pub fn init_https(root_folder: &Path) -> std::io::Result<openssl::ssl::SslAcceptorBuilder> {
    let mut builder =
        openssl::ssl::SslAcceptor::mozilla_intermediate(openssl::ssl::SslMethod::tls())
            .map_err(|err| std::io::Error::other(err.to_string()))?;
    builder
        .set_private_key_file(
            root_folder.join("ssl/key.pem"),
            openssl::ssl::SslFiletype::PEM,
        )
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    builder
        .set_certificate_chain_file(root_folder.join("ssl/cert.pem"))
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    Ok(builder)
}

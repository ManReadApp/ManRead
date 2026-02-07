use std::path::Path;

pub fn init_https(root_folder: &Path) -> openssl::ssl::SslAcceptorBuilder {
    let mut builder =
        openssl::ssl::SslAcceptor::mozilla_intermediate(openssl::ssl::SslMethod::tls())
            .expect("Couldnt initialize SslAcceptor");
    builder
        .set_private_key_file(
            root_folder.join("ssl/key.pem"),
            openssl::ssl::SslFiletype::PEM,
        )
        .expect("File does not exist");
    builder
        .set_certificate_chain_file(root_folder.join("ssl/cert.pem"))
        .expect("File does not exist");
    builder
}

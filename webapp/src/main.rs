use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;
use webapp::configuration::get_configuration;
use webapp::create_app;
use webapp::errors::Error;

fn bind_address(host: &str, port: u16) -> Result<SocketAddr, Error> {
    let host = IpAddr::from_str(host)?;
    Ok(SocketAddr::from((host, port)))
}

#[tokio::main]
async fn main() {
    let configuration = get_configuration().expect("Failed to read configuration");
    let addr = bind_address(
        &configuration.application.host,
        configuration.application.port,
    )
    .expect("Failed to create socket address");
    let app = create_app().expect("Failed to start server");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

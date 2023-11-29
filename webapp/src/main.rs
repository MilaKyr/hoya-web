use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::MissedTickBehavior::Skip;
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

    let (app, app_state) = create_app().expect("Failed to start server");
    let mut interval = tokio::time::interval(Duration::from_secs(configuration.parsing_delay));
    interval.set_missed_tick_behavior(Skip);
    let task = tokio::task::spawn(async move {
        loop {
            interval.tick().await;
            let _ = app_state.parse().await;
            // TODO add to logging
        }
    });
    task.await.expect("Failed to parse data");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

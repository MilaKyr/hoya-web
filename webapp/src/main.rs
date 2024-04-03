use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::MissedTickBehavior::Skip;
use webapp::configuration::get_configuration;
use webapp::create_app;
use webapp::errors::Error;

async fn bind_address(host: &str, port: u16) -> Result<TcpListener, Error> {
    let address = format!("{}:{}", host, port);
    let listener = TcpListener::bind(address).await?;
    Ok(listener)
}

#[tokio::main]
async fn main() {
    let configuration = get_configuration().expect("Failed to read configuration");
    let listener = bind_address(
        &configuration.application.host,
        configuration.application.port,
    )
    .await
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
    axum::serve(listener, app).await.unwrap();
}

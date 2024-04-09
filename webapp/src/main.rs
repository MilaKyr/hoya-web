use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::MissedTickBehavior::Skip;
use webapp::configuration::get_configuration;
use webapp::create_app;
use webapp::db::Database;

#[tokio::main]
async fn main() {
    let configuration = get_configuration().expect("Failed to read configuration");
    let listener = TcpListener::bind(&configuration.application.bind_address())
                                         .await
                                         .expect("Failed to create socket address");

    let db = Database::try_from(&configuration.database)
        .await
        .expect("Failed to start DB");
    let (app, app_state) = create_app(db).expect("Failed to start server");
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

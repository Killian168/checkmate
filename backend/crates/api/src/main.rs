use lambda_http::{run, tracing, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    let state = api::AppState::new().await;
    let app = api::create_app(state);

    run(app).await
}

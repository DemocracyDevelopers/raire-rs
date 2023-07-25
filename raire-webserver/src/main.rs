use axum::{
    routing::{post},
    http::StatusCode,
    Json, Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_static_file_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let serve_dir = ServeDir::new("WebContent");

    // build our application with a route
    let app = Router::new()
        // `POST /raire` goes to `raire`
        .route("/raire", post(raire))
        .nest_service("/",serve_dir);


    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


async fn raire(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(problem): Json<raire::RaireProblem>,
) -> (StatusCode, Json<raire::RaireSolution>) {
    let solution = problem.solve();
    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::OK, Json(solution))
}

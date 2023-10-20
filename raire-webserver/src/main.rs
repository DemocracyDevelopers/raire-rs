// Copyright 2023 Andrew Conway.
// Based on software (c) Michelle Blom in C++ https://github.com/michelleblom/audit-irv-cp/tree/raire-branch
// documented in https://arxiv.org/pdf/1903.08804.pdf
//
// This file is part of raire-rs.
// raire-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
// raire-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
// You should have received a copy of the GNU Affero General Public License along with ConcreteSTV.  If not, see <https://www.gnu.org/licenses/>.



use axum::{
    routing::{post},
    http::StatusCode,
    Json, Router,
};
use std::net::{IpAddr, SocketAddr};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
/// A server that performs the RAIRE algorithm as a webservice
struct CliOptions {
    /// The socket to listen on. Default is 3000.
    #[arg(short, long)]
    socket : Option<u16>,

    /// The IP address to listen to. Default is 127.0.0.1
    #[arg(short, long)]
    ip : Option<IpAddr>,

}


#[tokio::main]
async fn main() {
    let args = CliOptions::parse();
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
    let ip = args.ip.unwrap_or_else(||[127, 0, 0, 1].into());
    let addr = SocketAddr::from((ip, args.socket.unwrap_or(3000)));
    println!("listening on {}", addr);
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

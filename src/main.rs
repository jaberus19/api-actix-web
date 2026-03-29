use actix_web::{web, App, HttpServer, Responder, HttpRequest, Error};
use actix_ws::Message;
use std::sync::Arc;
use tokio::sync::Mutex;

mod server;
mod models;
mod session;

use crate::server::WashServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. On crée l'instance du serveur (le Lobby)
    // On l'enveloppe dans un Mutex pour qu'il soit modifiable en toute sécurité
    let wash_server = web::Data::new(Mutex::new(WashServer::new()));

    println!("🚀 Serveur d'autolavage démarré sur http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(wash_server.clone()) // On partage le serveur avec toutes les routes
            .route("/ws", web::get().to(routes::ws_index)) // La route de connexion
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
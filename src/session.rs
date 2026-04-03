use tokio::sync::{mpsc, Mutex}; 
use std::sync::Arc;             
use crate::messages::{WsMessage, UserRole};
use crate::server::WashServer;

pub async fn start_session(
    id: usize,
    role: UserRole,
    srv: Arc<Mutex<WashServer>>,
    mut session: actix_ws::Session,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

    {
        let mut server = srv.lock().await;
        println!("User {} connected with role {:?}", id, role);
        server.connexion(id, tx, role);
    }

    while let Some(msg) = rx.recv().await {
        println!("[Client {}] New message from server: {:?}", id, msg);

        // On transforme le message en JSON
        if let Ok(text) = serde_json::to_string(&msg) {
            // On envoie le texte au navigateur
            let _ = session.text(text).await; 
        }
    }

    let mut server = srv.lock().await;
    server.deconnexion(id);
    println!("User {} disconnected", id);
}
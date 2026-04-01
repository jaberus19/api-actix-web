use tokio::sync::{mpsc, Mutex}; 
use core::sync;
use std::sync::Arc;              
use crate::models::{WsMessage, UserRole};
use crate::server::WashServer;

pub async fn start_session(
    id: usize,
    role: UserRole,
    srv: Arc<Mutex<WashServer>>,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

    {
        let mut server = srv.lock().await;
        println!("User {} connected with role {:?}", id, role);
        server.connexion(id, tx, role);
    }

    while let Some(msg) = rx.recv().await {
        println!("[Client {}] New message from server: {:?}", id, msg);
    }

    let mut server = srv.lock().await;
    server.deconnexion(id);
    println!("User {} disconnected", id);
}
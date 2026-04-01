mod models;
mod server;
mod session;

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::server::WashServer;
use crate::models::{UserRole, WsMessage};


#[actix_web::main]
async fn main()  {
   let server = Arc::new(Mutex::new(WashServer::new()));

   let srv_clone1 = Arc::clone(&server);
    tokio::spawn(async move {
        session::start_session(1, UserRole::Supervisor, srv_clone1).await;
    });

    let srv_clone2 = Arc::clone(&server);
    tokio::spawn(async move {
        session::start_session(2, UserRole::Client, srv_clone2).await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("--- BROADCAST TEST ---");
    let test_msg = WsMessage::Chat { 
        user: "System".to_string(), 
        text: "Welcome to the Car Wash service!".to_string() 
    };
    
    let srv_lock = server.lock().await;
    srv_lock.broadcast(test_msg);

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
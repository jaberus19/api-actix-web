mod models;
mod messages;
mod server;
mod session;

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use dotenvy::dotenv;
use std::env;

// On importe ton serveur
use server::WashServer;
use messages::{UserRole, WsMessage};

#[get("/clients")]
async fn get_clients(pool: web::Data<sqlx::PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, models::Client>("SELECT * FROM clients")
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(clients) => HttpResponse::Ok().json(clients),
        Err(_) => HttpResponse::InternalServerError().body("Erreur de lecture DB"),
    }
}

// --- NOUVELLE ROUTE : La porte d'entrée WebSocket ---
#[get("/ws")]
async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Arc<Mutex<WashServer>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = rand::random::<usize>(); // Génère un ID unique pour la session
    
    // Upgrade the HTTP connection to a WebSocket connection
    let (response, ws_session, _msg_stream) = actix_ws::handle(&req, stream)?;
    
    // On appelle ta fonction dans session.rs !
    let srv_clone = srv.get_ref().clone();
    tokio::spawn(async move {
        session::start_session(id, UserRole::Admin, srv_clone, ws_session).await;
    });

    Ok(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL doit être défini");

    // 1. On crée le serveur de lavage UNE SEULE FOIS
    let wash_server = Arc::new(Mutex::new(WashServer::new()));

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Impossible de se connecter à la base de données");

    let srv_for_task = wash_server.clone();
    let pool_for_task = pool.clone();

    tokio::spawn(async move {
        // HashSet pour ne pas envoyer 50 notifications pour la même vente
        let mut sent_notifications = std::collections::HashSet::new();
        let mut last_sale_statuses: HashMap<i32, String> = HashMap::new();
        let mut last_summary: Option<(i64, i64, i64, i64, i64)> = None;

        loop {
            println!("🔍 [DEBUG] Le robot vérifie la table sales..."); 

            let result = sqlx::query_as::<_, models::Sale>(
                "SELECT 
                    saleid, 
                    clientid, 
                    vehicleid, 
                    paymentmethodid, 
                    statussale::TEXT,    -- On transforme l'ENUM en TEXT
                    stateuswashing::TEXT, -- On transforme l'ENUM en TEXT
                    saledate, 
                    initial_state 
                FROM sales"
            )
            .fetch_all(&pool_for_task)
            .await;

            match result {
                Ok(sales) => {
                    let mut current_statuses: HashMap<i32, String> = HashMap::new();

                    if sales.is_empty() {
                        println!("ℹ️ [DEBUG] Rien de neuf.");
                    } else {
                        let server = srv_for_task.lock().await;

                        let mut pending = 0_i64;
                        let mut in_progress = 0_i64;
                        let mut finished = 0_i64;
                        let mut delivered = 0_i64;
                        let mut canceled = 0_i64;

                        for sale in sales {
                            current_statuses.insert(sale.id, sale.stateuswashing.clone());

                            match sale.stateuswashing.as_str() {
                                "En espera" => pending += 1,
                                "En proceso" => in_progress += 1,
                                "Terminado" => finished += 1,
                                "Entregado" => delivered += 1,
                                "Cancelado" => canceled += 1,
                                _ => {}
                            }

                            if let Some(previous) = last_sale_statuses.get(&sale.id) {
                                if previous != &sale.stateuswashing {
                                    server.broadcast(WsMessage::SupervisorSaleStateChanged {
                                        sale_id: sale.id,
                                        previous_status: previous.clone(),
                                        current_status: sale.stateuswashing.clone(),
                                    });
                                }
                            }

                            if !sent_notifications.contains(&sale.id) {
                                // 1. Notification Terminal
                                println!("📢 [SUCCÈS] Notification pour la vente ID: {}", sale.id);
                                
                                // 2. Notification WebSocket (Temps réel !)
                                let msg = WsMessage::WashStatusUpdate {
                                    sale_id: sale.id,
                                    plate: "Véhicule".to_string(),
                                    new_status: sale.stateuswashing.clone(),
                                };
                                server.broadcast(msg);
                                
                                sent_notifications.insert(sale.id);
                            }
                        }

                        let summary = (pending, in_progress, finished, delivered, canceled);
                        if last_summary != Some(summary) {
                            server.broadcast(WsMessage::SupervisorSalesSummary {
                                pending,
                                in_progress,
                                finished,
                                delivered,
                                canceled,
                            });
                            last_summary = Some(summary);
                        }
                    }

                    last_sale_statuses = current_statuses;
                },
                Err(e) => println!("❌ [ERREUR] SQL : {:?}", e),
            }

            sleep(Duration::from_secs(5)).await;
        }
        
    });

    println!("✅ Serveur et surveillance DB activés !");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            // 2. On partage le serveur de lavage avec l'App
            .app_data(web::Data::new(wash_server.clone())) 
            .service(get_clients)
            .service(websocket_route) // 3. On enregistre la route WS
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

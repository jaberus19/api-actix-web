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

// --- Ruta WebSocket: acepta ?role=admin|supervisor|cliente|cajero ---
#[get("/ws")]
async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Arc<Mutex<WashServer>>>,
) -> Result<HttpResponse, actix_web::Error> {
    let id = rand::random::<usize>();

    // Leer el rol desde el query string: /ws?role=cajero
    let role = req
        .query_string()
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(k, _)| k == &"role")
        .map(|(_, v)| v.to_lowercase())
        .and_then(|v| match v.as_str() {
            "admin"      => Some(UserRole::Admin),
            "supervisor" => Some(UserRole::Supervisor),
            "cliente"    => Some(UserRole::Cliente),
            "cajero"     => Some(UserRole::Cajero),
            _            => None,
        })
        .unwrap_or(UserRole::Supervisor); // valor por defecto

    // Upgrade the HTTP connection to a WebSocket connection
    let (response, ws_session, _msg_stream) = actix_ws::handle(&req, stream)?;

    let srv_clone = srv.get_ref().clone();
    tokio::spawn(async move {
        session::start_session(id, role, srv_clone, ws_session).await;
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

    // ================================================================
    // FUNCIONES AUXILIARES DE BROADCAST POR ROL
    // ================================================================

    /// Envía un mensaje solo a los Supervisores conectados.
    fn broadcast_supervisor(server: &WashServer, msg: WsMessage) {
        server.broadcast_to_role(msg, UserRole::Supervisor);
    }

    /// Envía un mensaje a Supervisores y Cajeros.
    fn broadcast_supervisor_and_cashier(server: &WashServer, msg: WsMessage) {
        server.broadcast_to_roles(msg, &[UserRole::Supervisor, UserRole::Cajero]);
    }

    /// Envía un mensaje solo a los Cajeros conectados.
    fn broadcast_cashier(server: &WashServer, msg: WsMessage) {
        server.broadcast_to_role(msg, UserRole::Cajero);
    }

    /// Envía un mensaje solo a los Clientes conectados.
    fn broadcast_client(server: &WashServer, msg: WsMessage) {
        server.broadcast_to_role(msg, UserRole::Cliente);
    }

    /// Envía un mensaje solo a los Admins conectados.
    fn broadcast_admin(server: &WashServer, msg: WsMessage) {
        server.broadcast_to_role(msg, UserRole::Admin);
    }

    tokio::spawn(async move {
        // HashSet pour ne pas envoyer 50 notifications pour la même vente
         let mut sent_notifications = std::collections::HashSet::new();
         let mut last_sale_statuses: HashMap<i32, models::WashingStatus> = HashMap::new();
         let mut last_summary: Option<(i64, i64, i64, i64)> = None;

         loop {
             println!("🔍 [DEBUG] Le robot vérifie la table sales...");

             let result = sqlx::query_as::<_, models::Sale>(
                 "SELECT
                     \"saleId\" AS saleid,
                     \"clientId\" AS clientid,
                     \"vehicleId\" AS vehicleid,
                     \"paymentMethodId\" AS paymentmethodid,
                     \"statusSale\" AS statussale,
                     \"statusWashing\" AS stateuswashing,
                     \"saleDate\" AS saledate,
                     \"initialState\" AS initial_state
                 FROM sales"
             )
             .fetch_all(&pool_for_task)
             .await;

             match result {
                 Ok(sales) => {
                     let mut current_statuses: HashMap<i32, models::WashingStatus> = HashMap::new();

                     if sales.is_empty() {
                         println!("ℹ️ [DEBUG] Rien de neuf.");
                     } else {
                         let server = srv_for_task.lock().await;

                           let mut pending = 0_i64;
                           let mut in_progress = 0_i64;
                           let mut completed = 0_i64;
                           let mut canceled = 0_i64;

                           for sale in sales {
                               current_statuses.insert(sale.id, sale.stateuswashing.clone());

                               match sale.stateuswashing {
                                   models::WashingStatus::EnEspera => pending += 1,
                                   models::WashingStatus::EnProceso => in_progress += 1,
                                   models::WashingStatus::Completado => completed += 1,
                                   models::WashingStatus::Cancelado => canceled += 1,
                               }

                             // --- CAMBIO DE ESTADO: notificar a Supervisor ---
                             if let Some(previous) = last_sale_statuses.get(&sale.id) {
                                 if previous != &sale.stateuswashing {
                                     // SUPERVISOR_SALE_STATE_CHANGED → Supervisor
                                     broadcast_supervisor(&server, WsMessage::SupervisorSaleStateChanged {
                                         sale_id: sale.id,
                                         estado_anterior: previous.display().to_string(),
                                         estado_actual: sale.stateuswashing.display().to_string(),
                                     });

                                     // Si pasó a "Completado" → notificar al Cliente
                                     if let models::WashingStatus::Completado = sale.stateuswashing {
                                         broadcast_client(&server, WsMessage::VehicleReady {
                                             placa: format!("Vehículo #{}", sale.vehicleid),
                                             nombre_servicio: "Lavado completado".to_string(),
                                         });
                                     }

                                     // Si pasó a "En proceso" → notificar al Cliente
                                     if let models::WashingStatus::EnProceso = sale.stateuswashing {
                                         broadcast_client(&server, WsMessage::VehicleInProgress {
                                             placa: format!("Vehículo #{}", sale.vehicleid),
                                             nombre_servicio: "Lavado en proceso".to_string(),
                                         });
                                     }

                                     // Si pasó a "Cancelado" → notificar al Cliente
                                     if let models::WashingStatus::Cancelado = sale.stateuswashing {
                                         broadcast_client(&server, WsMessage::VehicleCanceled {
                                             placa: format!("Vehículo #{}", sale.vehicleid),
                                             nombre_servicio: "Lavado".to_string(),
                                             motivo: "Cancelado por el supervisor".to_string(),
                                         });
                                     }
                                 }
                             }

                             if !sent_notifications.contains(&sale.id) {
                                 // 1. Notificación en terminal
                                 println!("📢 [INFO] Nueva venta detectada ID: {}", sale.id);

                                 // 2. WASH_STATUS_UPDATE → Supervisor
                                 broadcast_supervisor(&server, WsMessage::WashStatusUpdate {
                                     sale_id: sale.id,
                                     placa: format!("Vehículo #{}", sale.vehicleid),
                                     nuevo_estado: sale.stateuswashing.display().to_string(),
                                 });

                                 // 3. NEW_SALE_CREATED → Supervisor + Cajero
                                 broadcast_supervisor_and_cashier(&server, WsMessage::NewSale {
                                     sale_id: sale.id,
                                     tipo_vehiculo: "Vehículo".to_string(),
                                     servicios: vec!["Lavado".to_string()],
                                 });

                                 // 4. VEHICLE_PENDING → Cliente (si está en espera)
                                 if let models::WashingStatus::EnEspera = sale.stateuswashing {
                                     broadcast_client(&server, WsMessage::VehiclePending {
                                         placa: format!("Vehículo #{}", sale.vehicleid),
                                         nombre_servicio: "Lavado".to_string(),
                                     });
                                 }

                                 // 5. PENDING_PAYMENT_REMINDER → Cajero (si la venta está sin pagar)
                                 if let models::StatusPayments::Waiting = sale.statussale {
                                     broadcast_cashier(&server, WsMessage::PendingPaymentReminder {
                                         sale_id: sale.id,
                                         nombre_cliente: format!("Cliente #{}", sale.clientid),
                                         monto_pendiente: 0.0,
                                         dias_pendiente: 0,
                                     });
                                 }

                                 sent_notifications.insert(sale.id);
                             }
                         }

                          let summary = (pending, in_progress, completed, canceled);
                          if last_summary != Some(summary) {
                              // SUPERVISOR_SALES_SUMMARY → solo Supervisores
                              broadcast_supervisor(&server, WsMessage::SupervisorSalesSummary {
                                  pendientes: pending,
                                  en_proceso: in_progress,
                                  completadas: completed,
                                  canceladas: canceled,
                              });

                              // CASHIER_DAILY_SUMMARY → solo Cajeros
                              broadcast_cashier(&server, WsMessage::CashierDailySummary {
                                  total_recaudado: 0.0,
                                  cantidad_pendientes: pending,
                                  cantidad_transacciones: (pending + in_progress + completed + canceled),
                              });

                              // UNPAID_SALES_ALERT → Supervisor + Cajero
                              let ventas_sin_pagar = pending; // aproximación: pendientes = sin pagar
                              broadcast_supervisor_and_cashier(&server, WsMessage::UnpaidSalesAlert {
                                  cantidad_sin_pagar: ventas_sin_pagar,
                                  monto_total_pendiente: 0.0, // se calcularía desde tabla de pagos
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

   // ================================================================
   // TAREA 2: Monitoreo de stock y combos (cada 30 segundos)
   // ================================================================
   let srv_for_stock = wash_server.clone();
   let _pool_for_stock = pool.clone();

   tokio::spawn(async move {
       let mut last_stock_alerts: std::collections::HashSet<String> = std::collections::HashSet::new();
       let mut last_expired_combos: std::collections::HashSet<String> = std::collections::HashSet::new();
       let mut last_expiring_combos: std::collections::HashSet<String> = std::collections::HashSet::new();

       loop {
           println!("🔍 [STOCK] Verificando inventario y combos...");

           let server = srv_for_stock.lock().await;

           // ── ALERTA DE STOCK CRÍTICO ──────────────────────────
           // TODO: cuando exista la tabla `inventory` o `products`,
           // reemplazar esta consulta placeholder por:
           //   SELECT nombre_producto, stock_actual, stock_minimo
           //   FROM inventory WHERE stock_actual <= stock_minimo
           let productos_bajo_stock: Vec<(String, f64, f64)> = vec![
               // ("Shampoo", 2.0, 5.0),  // ejemplo
           ];

           for (nombre, actual, minimo) in productos_bajo_stock {
               let clave = format!("stock_{}", nombre);
               if !last_stock_alerts.contains(&clave) {
                   broadcast_supervisor(&server, WsMessage::StockAlert {
                       nombre_producto: nombre.clone(),
                       stock_actual: actual,
                       stock_minimo: minimo,
                   });
                   broadcast_admin(&server, WsMessage::StockAlert {
                       nombre_producto: nombre.clone(),
                       stock_actual: actual,
                       stock_minimo: minimo,
                   });
                   last_stock_alerts.insert(clave);
                   println!("⚠️  [STOCK] Alerta enviada: {} (stock={}, mínimo={})", nombre, actual, minimo);
               }
           }

           // Limpiar alertas de productos que ya no están bajo stock
           last_stock_alerts.clear();

           // ── ALERTA DE COMBOS VENCIDOS ────────────────────────
           // TODO: cuando exista la tabla `combos` o `promotions`,
           // reemplazar esta consulta placeholder por:
           //   SELECT nombre_combo, fecha_vencimiento
           //   FROM combos WHERE fecha_vencimiento < NOW()
           let combos_vencidos: Vec<String> = vec![
               // "Combo Básico Vencido".to_string(),  // ejemplo
           ];

           for combo in &combos_vencidos {
               if !last_expired_combos.contains(combo) {
                   broadcast_supervisor(&server, WsMessage::ExpiredCombosAlert {
                       cantidad_vencidos: combos_vencidos.len() as i64,
                       combos: combos_vencidos.clone(),
                   });
                   broadcast_admin(&server, WsMessage::ExpiredCombosAlert {
                       cantidad_vencidos: combos_vencidos.len() as i64,
                       combos: combos_vencidos.clone(),
                   });
                   last_expired_combos.insert(combo.clone());
               }
           }
           last_expired_combos.clear();

           // ── ALERTA DE COMBOS PRÓXIMOS A VENCER ───────────────
           // TODO: cuando exista la tabla `combos`,
           // reemplazar esta consulta placeholder por:
           //   SELECT nombre_combo, fecha_vencimiento
           //   FROM combos
           //   WHERE fecha_vencimiento BETWEEN NOW() AND NOW() + INTERVAL '3 days'
           let combos_por_vencer: Vec<String> = vec![
               // "Combo Premium".to_string(),  // ejemplo
           ];
           let dias_restantes: i32 = 3; // ejemplo: alertar a 3 días de vencer

           for combo in &combos_por_vencer {
               if !last_expiring_combos.contains(combo) {
                   broadcast_supervisor(&server, WsMessage::ExpiringCombosAlert {
                       cantidad_por_vencer: combos_por_vencer.len() as i64,
                       dias_restantes,
                       combos: combos_por_vencer.clone(),
                   });
                   broadcast_admin(&server, WsMessage::ExpiringCombosAlert {
                       cantidad_por_vencer: combos_por_vencer.len() as i64,
                       dias_restantes,
                       combos: combos_por_vencer.clone(),
                   });
                   last_expiring_combos.insert(combo.clone());
               }
           }
           last_expiring_combos.clear();

           drop(server);
           sleep(Duration::from_secs(30)).await;
       }
   });

   println!("✅ Servicio de monitoreo de stock y combos activado !");

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

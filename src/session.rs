use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use crate::messages::{WsMessage, UserRole};
use crate::server::WashServer;

/// Inicia una sesión WebSocket para un usuario con un rol específico.
///
/// # Roles soportados
/// | Rol         | Recibe notificaciones de                     |
/// |-------------|----------------------------------------------|
/// | `Admin`     | `URGENT_PURCHASE`, `STOCK_ALERT`             |
/// | `Supervisor`| `WASH_STATUS_UPDATE`, `SUPERVISOR_*`, `NEW_SALE_CREATED` |
/// | `Cajero`    | `NEW_SALE_CREATED`, `PAYMENT_RECEIVED`, `PENDING_PAYMENT_REMINDER`, `CASHIER_DAILY_SUMMARY` |
/// | `Cliente`   | `VEHICLE_IN_PROGRESS`, `VEHICLE_READY`, `ASSIGNED_LAVADOR` |
pub async fn start_session(
    id: usize,
    role: UserRole,
    srv: Arc<Mutex<WashServer>>,
    mut session: actix_ws::Session,
) {
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

    {
        let mut server = srv.lock().await;
        println!("🔌 User {} connected with role {:?}", id, role);
        server.connexion(id, tx, role.clone());
    }

    while let Some(msg) = rx.recv().await {
        // Filtrado por rol: cada rol solo recibe los mensajes que le corresponden.
        // El filtrado principal se hace en el servidor (broadcast_to_role),
        // pero aquí agregamos una capa extra de validación por seguridad.
        if should_receive_message(&role, &msg) {
            println!("[Client {} | {:?}] ← {:?}", id, role, msg);

            // Transformamos el mensaje en JSON
            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = session.text(text).await;
            }
        } else {
            println!("[Client {} | {:?}] ✋ Mensaje filtrado (no corresponde al rol): {:?}", id, role, msg);
        }
    }

    let mut server = srv.lock().await;
    server.deconnexion(id);
    println!("🔌 User {} disconnected", id);
}

/// Determina si un mensaje WS debe ser entregado a un rol dado.
///
/// Esta función actúa como **filtro de seguridad** en el lado del servidor.
/// El broadcast selectivo se hace en `WashServer::broadcast_to_role`, pero
/// esta capa adicional evita fugas de información por errores de routing.
fn should_receive_message(role: &UserRole, msg: &WsMessage) -> bool {
    match msg {
        // ── Mensajes globales (todos los roles) ──────────────────
        WsMessage::Chat { .. } | WsMessage::Ping => true,

        // ── Mensajes de Supervisor ───────────────────────────────
        WsMessage::WashStatusUpdate { .. }
        | WsMessage::SupervisorSalesSummary { .. }
        | WsMessage::SupervisorSaleStateChanged { .. }
        | WsMessage::NewSale { .. }
        | WsMessage::UnpaidSalesAlert { .. }
        | WsMessage::ExpiredCombosAlert { .. }
        | WsMessage::ExpiringCombosAlert { .. } => {
            matches!(role, UserRole::Admin | UserRole::Supervisor | UserRole::Cajero)
        }

        // ── Mensajes de Cajero ──────────────────────────────────
        WsMessage::PaymentReceived { .. }
        | WsMessage::PendingPaymentReminder { .. }
        | WsMessage::CashierDailySummary { .. } => {
            matches!(role, UserRole::Admin | UserRole::Cajero)
        }

        // ── Mensajes de Cliente ─────────────────────────────────
        WsMessage::VehiclePending { .. }
        | WsMessage::VehicleInProgress { .. }
        | WsMessage::VehicleReady { .. }
        | WsMessage::VehicleCanceled { .. }
        | WsMessage::AssignedLavador { .. } => {
            matches!(role, UserRole::Cliente)
        }

        // ── Mensajes de Empleado / Lavador ──────────────────────
        WsMessage::NewAssignment { .. } => {
            matches!(role, UserRole::Supervisor)
        }

        // ── Mensajes de Admin ───────────────────────────────────
        WsMessage::StockAlert { .. }
        | WsMessage::UrgentPurchase { .. } => {
            matches!(role, UserRole::Admin)
        }
    }
}
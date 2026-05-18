use serde::{Deserialize, Serialize};

// ============================================================
// ROLES DEL SISTEMA (4 roles)
// ============================================================
/// Administrador total del sistema.
/// Gestiona usuarios, precios, stocks, configuraciones globales.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserRole {
    /// Gestión total del sistema: precios, stocks, usuarios, config.
    Admin,
    /// Encargado operativo: asigna lavadores, cambia estados, supervisa ventas.
    Supervisor,
    /// Cliente final: dueño del vehículo, recibe notificaciones de su lavado.
    Cliente,
    /// Cajero: procesa pagos, genera facturas, gestiona cobros pendientes.
    Cajero,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum WsMessage {

    Chat { usuario: String, texto: String },

    // El supervisor cambia el estado de un lavado
    #[serde(rename = "WASH_STATUS_UPDATE")]
    WashStatusUpdate {
        sale_id: i32,
        placa: String,
        nuevo_estado: String, // ej: "En proceso"
    },

    // Alerta de stock crítico (para Admin y Supervisor)
    #[serde(rename = "STOCK_ALERT")]
    StockAlert {
        nombre_producto: String,
        stock_actual: f64,
        stock_minimo: f64,
    },

    // Nueva venta creada (notifica a Supervisor y Cajero)
    #[serde(rename = "NEW_SALE_CREATED")]
    NewSale {
        sale_id: i32,
        tipo_vehiculo: String,
        servicios: Vec<String>,
    },

    // Resumen de ventas por estado para el dashboard del Supervisor
    #[serde(rename = "SUPERVISOR_SALES_SUMMARY")]
    SupervisorSalesSummary {
        pendientes: i64,
        en_proceso: i64,
        completadas: i64,
        canceladas: i64,
    },

    // Cambio de estado de una venta detectado entre ciclos de polling
    #[serde(rename = "SUPERVISOR_SALE_STATE_CHANGED")]
    SupervisorSaleStateChanged {
        sale_id: i32,
        estado_anterior: String,
        estado_actual: String,
    },

    // Alerta de compra urgente / factura pendiente de pago a proveedor
    #[serde(rename = "URGENT_PURCHASE")]
    UrgentPurchase {
        nombre_proveedor: String,
        numero_factura: String,
        monto_pendiente: f64,
    },

    // Alerta de ventas sin pagar (Supervisor y Cajero)
    #[serde(rename = "UNPAID_SALES_ALERT")]
    UnpaidSalesAlert {
        cantidad_sin_pagar: i64,
        monto_total_pendiente: f64,
    },

    // Alerta de combos vencidos (Supervisor y Admin)
    #[serde(rename = "EXPIRED_COMBOS_ALERT")]
    ExpiredCombosAlert {
        cantidad_vencidos: i64,
        combos: Vec<String>,
    },

    // Alerta de combos próximos a vencer (Supervisor y Admin)
    #[serde(rename = "EXPIRING_COMBOS_ALERT")]
    ExpiringCombosAlert {
        cantidad_por_vencer: i64,
        dias_restantes: i32,
        combos: Vec<String>,
    },

    // ================================================================
    // MENSAJES PARA CAJERO
    // ================================================================

    /// Pago recibido / factura generada (notifica a Cajero y Cliente).
    #[serde(rename = "PAYMENT_RECEIVED")]
    PaymentReceived {
        sale_id: i32,
        nombre_cliente: String,
        monto: f64,
        metodo_pago: String,
    },

    /// Recordatorio de pago pendiente para el Cajero.
    #[serde(rename = "PENDING_PAYMENT_REMINDER")]
    PendingPaymentReminder {
        sale_id: i32,
        nombre_cliente: String,
        monto_pendiente: f64,
        dias_pendiente: i32,
    },

    /// Resumen de cobros del día para el Cajero.
    #[serde(rename = "CASHIER_DAILY_SUMMARY")]
    CashierDailySummary {
        total_recaudado: f64,
        cantidad_pendientes: i64,
        cantidad_transacciones: i64,
    },

    // Notificación al Cliente: su vehículo está pendiente de lavado
    #[serde(rename = "VEHICLE_PENDING")]
    VehiclePending {
        placa: String,
        nombre_servicio: String,
    },

    // Notificación al Cliente: su vehículo entró en proceso de lavado
    #[serde(rename = "VEHICLE_IN_PROGRESS")]
    VehicleInProgress {
        placa: String,
        nombre_servicio: String,
    },

    // Notificación al Cliente: su vehículo está listo / lavado terminado
    #[serde(rename = "VEHICLE_READY")]
    VehicleReady {
        placa: String,
        nombre_servicio: String,
    },

    // Notificación al Cliente: su lavado fue cancelado
    #[serde(rename = "VEHICLE_CANCELED")]
    VehicleCanceled {
        placa: String,
        nombre_servicio: String,
        motivo: String,
    },

    // Notificación al Cliente: se le asignó un lavador
    #[serde(rename = "ASSIGNED_LAVADOR")]
    AssignedLavador {
        nombre_empleado: String,
        nombre_servicio: String,
    },

    // Notificación al Empleado: nueva asignación de lavado
    #[serde(rename = "NEW_ASSIGNMENT")]
    NewAssignment {
        nombre_servicio: String,
        placa: String,
    },

    // Ping / heartbeat para validar la conexión
    #[serde(rename = "PING")]
    Ping,
}

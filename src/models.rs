use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UserRole {
    Admin,
    Supervisor,
    Client,
}

// Dans ton fichier messages.rs (avec Serde)
#[derive(Deserialize)]
#[serde(tag = "cmd", content = "data")]
enum SupervisorCommand {
    AssignEmployee { vehicle_id: i32, employee_id: i32 },
    ChangeStatus { vehicle_id: i32, new_status: String },
}
/*// 2. NOUVEAU : Les commandes de l'Admin (ex: gestion des prix ou stocks)
#[derive(Deserialize, Debug)]
#[serde(tag = "cmd", content = "data")]
pub enum AdminCommand {
    UpdatePrice { service_id: i32, new_price: f64 },
    AddStock { product_id: i32, quantity: f64 },
}

// 3. NOUVEAU : Les commandes du Client (ex: demander de l'aide ou annuler)
#[derive(Deserialize, Debug)]
#[serde(tag = "cmd", content = "data")]
pub enum ClientCommand {
    RequestSupport { message: String },
} */

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum WsMessage {

    Chat { user: String, text: String },
    
    // Quand le superviseur change l'état d'un lavage (En espera -> En proceso)
    #[serde(rename = "WASH_STATUS_UPDATE")]
    WashStatusUpdate {
        sale_id: i32,
        plate: String,
        new_status: String, // ex: "En proceso"
    },

    // Quand le stock d'un produit est critique
    #[serde(rename = "STOCK_ALERT")]
    StockAlert {
        product_name: String,
        current_stock: f64,
        min_stock: f64,
    },

    // Quand une nouvelle vente est créée (Notification pour le superviseur)
    #[serde(rename = "NEW_SALE_CREATED")]
    NewSale {
        sale_id: i32,
        vehicle_type: String,
        services: Vec<String>,
    },

    // Un message simple pour tester la connexion (Ping/Pong)
    #[serde(rename = "PING")]
    Ping,
}
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

// --- LES ENUMS ---
#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "washing_status")]
pub enum WashingStatus {
    #[sqlx(rename = "W")] EnEspera,
    #[sqlx(rename = "I")] EnProceso,
    #[sqlx(rename = "D")] Completado,
    #[sqlx(rename = "C")] Cancelado,
}

impl WashingStatus {
    pub fn display(&self) -> &'static str {
        match self {
            WashingStatus::EnEspera => "En espera",
            WashingStatus::EnProceso => "En proceso",
            WashingStatus::Completado => "Completado",
            WashingStatus::Cancelado => "Cancelado",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "status_payments")]
pub enum StatusPayments {
    #[sqlx(rename = "W")] Waiting,
    #[sqlx(rename = "P")] Paid,
    #[sqlx(rename = "C")] Cancelled,
}

// --- LES TABLES (Mises à jour avec les vrais noms) ---

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Client {
    // On utilise rename car PostgreSQL met souvent les noms en minuscules en interne
    #[sqlx(rename = "clientId")] 
    pub id: i32,
    
    pub names: String,      // Doit être identique au SQL (names avec un 's')
    pub lastnames: String,  // Nouveau champ obligatoire
    
    #[sqlx(rename = "numberPhone")]
    pub phone: Option<String>, // Option car numberPhone peut être NULL dans ta DB
    
    pub ci: String,         // Nouveau champ obligatoire
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Sale {
    #[sqlx(rename = "saleId")] 
    #[serde(rename = "id")]
    pub id: i32,

    #[sqlx(rename = "clientId")] 
    pub clientid: i32,

    #[sqlx(rename = "vehicleId")] 
    pub vehicleid: i32,

    #[sqlx(rename = "paymentMethodId")] 
    pub paymentmethodid: i32,

    #[sqlx(rename = "statusSale")] 
    pub statussale: StatusPayments, 

    #[sqlx(rename = "statusWashing")] 
    pub stateuswashing: WashingStatus,

    #[sqlx(rename = "saleDate")] 
    pub saledate: NaiveDateTime,

    #[sqlx(rename = "initialState")] 
    pub initial_state: Option<String>,

    #[sqlx(rename = "invoiceNumber")] 
    pub invoice_number: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct SupervisorStatusCounter {
    pub status: String,
    pub total: i64,
}

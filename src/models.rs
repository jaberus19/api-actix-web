use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

// --- LES ENUMS ---
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "washing_status")]
pub enum WashingStatus {
    #[sqlx(rename = "En espera")] EnEspera,
    #[sqlx(rename = "En proceso")] EnProceso,
    #[sqlx(rename = "Terminado")] Terminado,
    #[sqlx(rename = "Entregado")] Entregado,
    #[sqlx(rename = "Cancelado")] Cancelado,
}

// --- LES TABLES (Mises à jour avec les vrais noms) ---

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Client {
    // On utilise rename car PostgreSQL met souvent les noms en minuscules en interne
    #[sqlx(rename = "clientid")] 
    pub id: i32,
    
    pub names: String,      // Doit être identique au SQL (names avec un 's')
    pub lastnames: String,  // Nouveau champ obligatoire
    
    #[sqlx(rename = "numberphone")]
    pub phone: Option<String>, // Option car numberPhone peut être NULL dans ta DB
    
    pub ci: String,         // Nouveau champ obligatoire
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Sale {
    #[sqlx(rename = "saleid")] 
    #[serde(rename = "id")]
    pub id: i32,

    pub clientid: i32,
    pub vehicleid: i32,
    pub paymentmethodid: i32,
    // Note : Pour les types ENUM comme status_payments, 
    // utilise String en Rust pour simplifier le test
    pub statussale: String, 
    pub stateuswashing: String,
    pub saledate: NaiveDateTime,
    pub initial_state: Option<String>, // Option car il peut être nul
}
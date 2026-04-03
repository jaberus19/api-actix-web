use serde::{Serialize, Deserialize};
use sqlx::FromRow;

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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Sale {
    #[sqlx(rename = "saleid")] // Vérifie si c'est 'saleId' dans la table de ton ami
    pub id: i32,
    pub total: f64,
    pub status: WashingStatus,
}
use actix_web::{get, web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};

// Esta estructura recibe los datos de la URL (?home=...&away=...)
#[derive(Deserialize)]
struct Info {
    home: String,
    away: String,
}

// Esta estructura se convertirá en el JSON de respuesta
#[derive(Serialize)]
struct PredictionResponse {
    home_team: String,
    home_prob: String,
    away_team: String,
    away_prob: String,
}

#[get("/predictions")]
async fn predict(info: web::Query<Info>) -> impl Responder {
    // Aquí iría tu lógica real o IA. Por ahora, simularemos probabilidades.
    let home_prob = 44.6;
    let away_prob = 55.4;

    let response = PredictionResponse {
        home_team: info.home.clone(),
        home_prob: format!("{}%", home_prob),
        away_team: info.away.clone(),
        away_prob: format!("{}%", away_prob),
    };

    HttpResponse::Ok().json(response) // .json() se encarga de todo el trabajo
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Servidor en http://127.0.0.1:8080");
    
    HttpServer::new(|| {
        App::new().service(predict)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

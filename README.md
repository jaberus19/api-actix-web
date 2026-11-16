# ⚡ AutoWash Real-Time Microservice (`api-actix-web`)

![Rust](https://img.shields.io/badge/Rust-2024_Edition-000000?style=flat-square&logo=rust&logoColor=white)
![Actix Web](https://img.shields.io/badge/Actix_Web-4.12-000000?style=flat-square&logo=rust&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-Async_Runtime-000000?style=flat-square&logo=rust&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-SQLx_0.7-4169E1?style=flat-square&logo=postgresql&logoColor=white)
![WebSockets](https://img.shields.io/badge/Protocol-WebSockets_%2B_REST-010101?style=flat-square&logo=socketdotio&logoColor=white)

Microservicio asíncrono de alto rendimiento desarrollado en **Rust** para la orquestación y despacho de notificaciones operativas en tiempo real mediante **WebSockets** y consultas **REST**, diseñado para entornos de gestión de autolavados.

---

### 🏗️ Arquitectura del Sistema

El servidor desacopla la recepción de conexiones WebSocket del monitoreo de cambios en base de datos mediante un *worker* asíncrono en **Tokio**, evitando bloqueos en el *event loop* y garantizando entrega segmentada por roles operativos.

```mermaid
flowchart LR
    DB[(PostgreSQL\nTabla: sales)] -->|Polling asíncrono\ntokio::spawn| Worker[Background Worker\nDetección & Deduplicación]
    Worker -->|broadcast_to_roles| Server[WashServer\nArc<Mutex<HashMap>>]
    Server -->|Canales MPSC| Admin[Rol: Admin / Supervisor]
    Server -->|Canales MPSC| Staff[Rol: Cajero]
    Server -->|Canales MPSC| Client[Rol: Cliente]
    HTTP[HTTP GET /clients] -->|SQLx Pool| DB
```

### 📂 Estructura del Proyecto

```text
src/
├── main.rs       # Inicialización de Actix/Tokio, PgPoolOptions, worker de polling y rutas
├── server.rs     # Gestor de sesiones concurrentes (HashMap + canales MPSC) y broadcast por rol
├── session.rs    # Ciclo de vida de conexiones individuales y streaming con actix-ws
├── messages.rs   # Definición de roles (Admin, Supervisor, Cliente, Cajero) y enum WsMessage
└── models.rs     # Estructuras mapeadas a PostgreSQL (Sale, Client, WashingStatus)
```

---

### 🔌 Endpoints y Protocolo WebSocket

#### Rutas Disponibles
| Método | Endpoint | Descripción |
| :--- | :--- | :--- |
| `GET` | `/clients` | Retorna el listado de clientes registrados en formato JSON desde PostgreSQL. |
| `GET` | `/ws?role=<rol>` | Inicia el *handshake* WebSocket. Roles válidos: `admin`, `supervisor`, `cliente`, `cajero`. |

#### Contrato de Mensajes (`WsMessage`)
Los eventos se serializan con etiquetado interno (`#[serde(tag = "type", content = "payload")]`) soportando más de 15 tipos de notificaciones operativas:

```json
{
  "type": "WASH_STATUS_UPDATE",
  "payload": {
    "sale_id": 1,
    "placa": "ABC-123",
    "nuevo_estado": "En proceso"
  }
}
```

---

### 🚀 Instalación y Ejecución Local

#### 1. Requisitos previos
- **Rust** (Edition 2024 / `cargo`)
- **PostgreSQL** en ejecución

#### 2. Configuración de entorno
Crea un archivo `.env` en la raíz del proyecto con la cadena de conexión a tu instancia de PostgreSQL:

```env
DATABASE_URL=postgres://usuario:clave@localhost:5432/autowash_db
```

#### 3. Inicializar datos de prueba y ejecutar
```bash
# Cargar script semilla para pruebas de transición de estados en WebSockets
psql -U postgres -d autowash_db -f scripts/sql/ws_seed_sales_test.sql

# Compilar y levantar el servidor en http://0.0.0.0:8080
cargo run
```

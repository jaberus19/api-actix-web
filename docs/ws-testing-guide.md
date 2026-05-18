# Guía completa de pruebas WS — 4 Roles

> **Roles del sistema:** `Admin` | `Supervisor` | `Cliente` | `Cajero`

Esta guía cubre el flujo completo de pruebas para notificaciones en tiempo real
de los **cuatro roles** del sistema: preparación, ejecución, resultados esperados
por rol, troubleshooting y limpieza.

---

## 1) Objetivo

Validar que el backend:

1. **No cambia estructura de tablas** (sin DDL).
2. Usa lectura de `sales` y emite eventos WS correctos.
3. Reacciona a cambios de `stateuswashing` con notificaciones en tiempo real.
4. **Separa correctamente los mensajes por rol** (ningún rol recibe eventos que no le corresponden).

---

## 2) Roles y eventos WS

### Matriz de responsabilidades

| Evento WS                    | Admin | Supervisor | Cajero | Cliente |
|------------------------------|:-----:|:----------:|:------:|:-------:|
| `WASH_STATUS_UPDATE`         |   ✓   |     ✓      |   ✓    |         |
| `SUPERVISOR_SALES_SUMMARY`   |       |     ✓      |        |         |
| `SUPERVISOR_SALE_STATE_CHANGED` |   |     ✓      |        |         |
| `NEW_SALE_CREATED`           |       |     ✓      |   ✓    |         |
| `STOCK_ALERT`                |   ✓   |            |        |         |
| `URGENT_PURCHASE`            |   ✓   |            |        |         |
| `PAYMENT_RECEIVED`           |   ✓   |            |   ✓    |         |
| `PENDING_PAYMENT_REMINDER`   |       |            |   ✓    |         |
| `CASHIER_DAILY_SUMMARY`      |       |            |   ✓    |         |
| `VEHICLE_IN_PROGRESS`        |       |            |        |    ✓    |
| `VEHICLE_READY`              |       |            |        |    ✓    |
| `ASSIGNED_LAVADOR`           |       |            |        |    ✓    |
| `NEW_ASSIGNMENT`             |       |     ✓      |        |         |
| `PING`                       |   ✓   |     ✓      |   ✓    |    ✓    |

### Descripción de cada rol

| Rol       | Responsabilidades principales                                                                 |
|-----------|-----------------------------------------------------------------------------------------------|
| `Admin`   | Gestión de usuarios, precios, stocks, configuraciones globales. Recibe `STOCK_ALERT`, `URGENT_PURCHASE`. |
| `Supervisor` | Asigna lavadores, cambia estados de ventas, supervisa el dashboard. Recibe todos los eventos de ventas. |
| `Cajero`  | Procesa pagos, genera facturas, gestiona cobros pendientes. Recibe `NEW_SALE_CREATED`, `PENDING_PAYMENT_REMINDER`, `CASHIER_DAILY_SUMMARY`. |
| `Cliente` | Dueño del vehículo. Recibe notificaciones de su lavado: `VEHICLE_IN_PROGRESS`, `VEHICLE_READY`, `ASSIGNED_LAVADOR`. |

---

## 3) Eventos WS a validar

### `WASH_STATUS_UPDATE`
- Se emite por cada `sale_id` detectado por primera vez en memoria.
- Estado enviado: valor real de `stateuswashing`.
- **Roles que lo reciben:** Admin, Supervisor, Cajero.

Ejemplo:

```json
{
  "type": "WASH_STATUS_UPDATE",
  "payload": {
    "sale_id": 90001,
    "plate": "Vehículo #1",
    "new_status": "En espera"
  }
}
```

### `SUPERVISOR_SALE_STATE_CHANGED`
- Se emite cuando una venta conocida cambia de estado entre ciclos de polling.
- **Roles que lo reciben:** Supervisor.

Ejemplo:

```json
{
  "type": "SUPERVISOR_SALE_STATE_CHANGED",
  "payload": {
    "sale_id": 90001,
    "previous_status": "En espera",
    "current_status": "En proceso"
  }
}
```

### `SUPERVISOR_SALES_SUMMARY`
- Se emite cuando cambia el agregado de conteos por estado.
- **Roles que lo reciben:** Supervisor.

Ejemplo:

```json
{
  "type": "SUPERVISOR_SALES_SUMMARY",
  "payload": {
    "pending": 1,
    "in_progress": 2,
    "completed": 1,
    "canceled": 0
  }
}
```

### `NEW_SALE_CREATED`
- Se emite cuando se detecta una nueva venta en la tabla `sales`.
- **Roles que lo reciben:** Supervisor, Cajero.

Ejemplo:

```json
{
  "type": "NEW_SALE_CREATED",
  "payload": {
    "sale_id": 90001,
    "vehicle_type": "Vehículo",
    "services": ["Lavado"]
  }
}
```

### `PENDING_PAYMENT_REMINDER` *(nuevo — Cajero)*
- Se emite por cada venta con `statussale = 'W'` (Waiting / sin pagar).
- **Roles que lo reciben:** Cajero.

Ejemplo:

```json
{
  "type": "PENDING_PAYMENT_REMINDER",
  "payload": {
    "sale_id": 90001,
    "client_name": "Cliente #1",
    "amount_due": 0.0,
    "days_pending": 0
  }
}
```

### `CASHIER_DAILY_SUMMARY` *(nuevo — Cajero)*
- Se emite cuando cambia el resumen de ventas (mismo trigger que `SUPERVISOR_SALES_SUMMARY`).
- **Roles que lo reciben:** Cajero.

Ejemplo:

```json
{
  "type": "CASHIER_DAILY_SUMMARY",
  "payload": {
    "total_collected": 0.0,
    "pending_count": 1,
    "transactions_count": 5
  }
}
```

### `VEHICLE_IN_PROGRESS` *(Cliente)*
- Se emite cuando una venta pasa a `En proceso`.
- **Roles que lo reciben:** Cliente.

### `VEHICLE_READY` *(Cliente)*
- Se emite cuando una venta pasa a `Completado`.
- **Roles que lo reciben:** Cliente.

### `STOCK_ALERT` *(Admin)*
- Alerta de stock crítico.
- **Roles que lo reciben:** Admin.

### `URGENT_PURCHASE` *(Admin)*
- Alerta de factura pendiente de pago a proveedor.
- **Roles que lo reciben:** Admin.

---

## 4) Pre-requisitos

1. PostgreSQL operativo.
2. Variables de entorno cargadas (ver `.env`).
3. Backend compilando correctamente.

Levantar backend:

```bash
cargo run
```

Endpoint WS:

`ws://127.0.0.1:8080/ws`

---

## 5) Estrategia de pruebas por rol

### 5.1 Supervisor

Conéctate como `UserRole::Supervisor` y valida:

1. `WASH_STATUS_UPDATE` por cada `sale_id` nuevo.
2. `SUPERVISOR_SALES_SUMMARY` con conteos correctos.
3. `SUPERVISOR_SALE_STATE_CHANGED` al ejecutar el script de transición.
4. `NEW_SALE_CREATED` al insertar la semilla.

### 5.2 Cajero

Conéctate como `UserRole::Cajero` y valida:

1. `NEW_SALE_CREATED` por cada venta de la semilla.
2. `PENDING_PAYMENT_REMINDER` por cada venta con `statussale = 'W'`.
3. `CASHIER_DAILY_SUMMARY` con el resumen de ventas.
4. **NO** debes recibir `SUPERVISOR_SALES_SUMMARY` ni `VEHICLE_READY`.

### 5.3 Cliente

Conéctate como `UserRole::Cliente` y valida:

1. Al ejecutar el script de transición, recibes `VEHICLE_IN_PROGRESS` (90001 → En proceso).
2. Recibes `VEHICLE_READY` (90002 → Completado).
3. **NO** debes recibir `SUPERVISOR_SALES_SUMMARY` ni `PENDING_PAYMENT_REMINDER`.

### 5.4 Admin

Conéctate como `UserRole::Admin` y valida:

1. Recibes `WASH_STATUS_UPDATE` y `NEW_SALE_CREATED`.
2. **NO** debes recibir `SUPERVISOR_SALES_SUMMARY` ni eventos de Cliente.
3. Para probar `STOCK_ALERT` y `URGENT_PURCHASE` se requiere integración con tablas de inventario/compras.

---

## 6) Scripts de prueba disponibles

- Semilla de datos: [`scripts/sql/ws_seed_sales_test.sql`](scripts/sql/ws_seed_sales_test.sql)
- Transiciones de estado: [`scripts/sql/ws_state_transition_test.sql`](scripts/sql/ws_state_transition_test.sql)

> Los scripts usan `saleid` 90001..90005 para no mezclarse con datos operativos.

---

## 7) Ejecución paso a paso

### ¿Qué necesitás?

| Terminal | Propósito |
|----------|-----------|
| **Terminal 1** | Backend Rust (`cargo run`) |
| **Terminal 2** | Cliente PostgreSQL (`psql`) para ejecutar los scripts SQL |
| **Navegador** | Consola del navegador (F12 → Console) para ver los mensajes WS |

---

### Paso A: Levantar el backend

En la **Terminal 1**:

```bash
cargo run
```

Deberías ver:
```
✅ Servicio de monitoreo de stock y combos activado !
✅ Serveur et surveillance DB activés !
```

El backend escucha en `http://127.0.0.1:8080` y el WebSocket en `ws://127.0.0.1:8080/ws`.

---

### Paso B: Conectar los 4 roles (consola del navegador)

Abrí la consola del navegador (F12 → pestaña **Console**) y pegá este código:

```js
// ── Supervisor ──────────────────────────────────────────────
const wsSupervisor = new WebSocket('ws://127.0.0.1:8080/ws?role=supervisor');
wsSupervisor.onopen    = () => console.log('✅ Supervisor conectado');
wsSupervisor.onmessage = e => console.log('[Supervisor]', JSON.parse(e.data));
wsSupervisor.onerror   = e => console.error('❌ Supervisor error', e);

// ── Cajero ──────────────────────────────────────────────────
const wsCajero = new WebSocket('ws://127.0.0.1:8080/ws?role=cajero');
wsCajero.onopen    = () => console.log('✅ Cajero conectado');
wsCajero.onmessage = e => console.log('[Cajero]', JSON.parse(e.data));
wsCajero.onerror   = e => console.error('❌ Cajero error', e);

// ── Cliente ─────────────────────────────────────────────────
const wsCliente = new WebSocket('ws://127.0.0.1:8080/ws?role=cliente');
wsCliente.onopen    = () => console.log('✅ Cliente conectado');
wsCliente.onmessage = e => console.log('[Cliente]', JSON.parse(e.data));
wsCliente.onerror   = e => console.error('❌ Cliente error', e);

// ── Admin ───────────────────────────────────────────────────
const wsAdmin = new WebSocket('ws://127.0.0.1:8080/ws?role=admin');
wsAdmin.onopen    = () => console.log('✅ Admin conectado');
wsAdmin.onmessage = e => console.log('[Admin]', JSON.parse(e.data));
wsAdmin.onerror   = e => console.error('❌ Admin error', e);
```

> **Importante:** El rol se pasa por el parámetro `?role=` en la URL.
> Valores válidos: `admin`, `supervisor`, `cliente`, `cajero`.
> Si no pasas el parámetro, el rol por defecto es `supervisor`.

---

### Paso C: Insertar semilla de datos

En la **Terminal 2** (PostgreSQL):

```sql
\i scripts/sql/ws_seed_sales_test.sql
```

Espera entre 5 y 10 segundos (el backend consulta la DB cada 5 s).

---

### Paso D: Verificar eventos iniciales

Mirá la consola del navegador. Deberías ver:

| Rol       | Eventos esperados                                                                 |
|-----------|-----------------------------------------------------------------------------------|
| Supervisor| `WASH_STATUS_UPDATE` × 5, `SUPERVISOR_SALES_SUMMARY` × 1, `NEW_SALE_CREATED` × 5 |
| Cajero    | `NEW_SALE_CREATED` × 5, `PENDING_PAYMENT_REMINDER` × 2, `CASHIER_DAILY_SUMMARY` × 1 |
| Cliente   | `VEHICLE_PENDING` × 2 (ventas 90001 y 90002 en `En espera`)                       |
| Admin     | `WASH_STATUS_UPDATE` × 5, `NEW_SALE_CREATED` × 5                                  |

---

### Paso E: Forzar transición de estados

En la **Terminal 2**:

```sql
\i scripts/sql/ws_state_transition_test.sql
```

Espera 5-10 segundos.

---

### Paso F: Verificar eventos de cambio

| Rol       | Eventos esperados                                                                 |
|-----------|-----------------------------------------------------------------------------------|
| Supervisor| `SUPERVISOR_SALE_STATE_CHANGED` × 3, `SUPERVISOR_SALES_SUMMARY` actualizado       |
| Cajero    | `CASHIER_DAILY_SUMMARY` actualizado, `UNPAID_SALES_ALERT` actualizado             |
| Cliente   | `VEHICLE_IN_PROGRESS` (90001), `VEHICLE_READY` (90002), `VEHICLE_CANCELED` (90003) |
| Admin     | `WASH_STATUS_UPDATE` por cada cambio                                              |

---

## 8) Matriz rápida (entrada → salida esperada)

| Acción en DB                          | Evento esperado                    | Roles que lo reciben       |
|---------------------------------------|------------------------------------|---------------------------|
| Insertar `saleid=90001` con `En espera` | `WASH_STATUS_UPDATE`             | Admin, Supervisor, Cajero |
| Insertar `saleid=90001` con `En espera` | `NEW_SALE_CREATED`               | Supervisor, Cajero        |
| Insertar `saleid=90001` con `statussale=W` | `PENDING_PAYMENT_REMINDER`      | Cajero                    |
| Cambiar 90001 a `En proceso`          | `SUPERVISOR_SALE_STATE_CHANGED`    | Supervisor                |
| Cambiar 90001 a `En proceso`          | `VEHICLE_IN_PROGRESS`              | Cliente                   |
| Cambiar 90002 a `Completado`          | `SUPERVISOR_SALE_STATE_CHANGED`    | Supervisor                |
| Cambiar 90002 a `Completado`          | `VEHICLE_READY`                    | Cliente                   |
| Cambiar 90003 a `Cancelado`           | `SUPERVISOR_SALE_STATE_CHANGED`    | Supervisor                |
| Cambiar varias ventas                 | `SUPERVISOR_SALES_SUMMARY`         | Supervisor                |
| Cambiar varias ventas                 | `CASHIER_DAILY_SUMMARY`            | Cajero                    |

---

## 9) Consultas SQL de verificación manual

Conteo por estado:

```sql
SELECT stateuswashing, COUNT(*)
FROM sales
GROUP BY stateuswashing
ORDER BY stateuswashing;
```

Rango de pruebas:

```sql
SELECT saleid, stateuswashing, statussale, saledate
FROM sales
WHERE saleid BETWEEN 90001 AND 90005
ORDER BY saleid;
```

---

## 10) Limpieza de datos de prueba

Cuando termines:

```sql
DELETE FROM sales
WHERE saleid BETWEEN 90001 AND 90005;
```

---

## 11) Troubleshooting

### No llegan eventos WS
1. Verifica que el backend siga corriendo (`cargo run` sin errores).
2. Confirma conexión al endpoint `ws://127.0.0.1:8080/ws`.
3. Revisa logs de backend para errores SQL.

### Llega summary pero no cambios de estado
1. Asegúrate de ejecutar el script de transición después de la semilla.
2. Verifica que realmente cambie `stateuswashing` (no mismo valor).

### Los conteos no cuadran
1. Ejecuta la consulta de `GROUP BY stateuswashing`.
2. Compara contra `payload` de `SUPERVISOR_SALES_SUMMARY`.

### Un rol recibe eventos que no le corresponden
1. Verifica la función [`should_receive_message()`](src/session.rs) en `session.rs`.
2. Verifica los métodos `broadcast_to_role` y `broadcast_to_roles` en [`server.rs`](src/server.rs).

---

## 12) Checklist final

- [ ] Se conectó WS sin errores (4 clientes, uno por rol).
- [ ] Se recibieron `WASH_STATUS_UPDATE` tras semilla (Admin, Supervisor, Cajero).
- [ ] Se recibió `SUPERVISOR_SALES_SUMMARY` inicial (Supervisor).
- [ ] Se recibió `CASHIER_DAILY_SUMMARY` inicial (Cajero).
- [ ] Se recibieron `PENDING_PAYMENT_REMINDER` por ventas sin pagar (Cajero).
- [ ] Se recibieron `SUPERVISOR_SALE_STATE_CHANGED` tras transición (Supervisor).
- [ ] Se recibieron `VEHICLE_IN_PROGRESS` / `VEHICLE_READY` (Cliente).
- [ ] Cada rol recibe **solo** los eventos que le corresponden.
- [ ] El resumen final coincide con SQL.
- [ ] Se limpiaron datos de prueba (90001..90005).

# API Actix Web — Guía Frontend para notificaciones (4 Roles)

> **Roles del sistema:** `admin` | `supervisor` | `cliente` | `cajero`

Este documento describe cómo el frontend puede conectarse al WebSocket del backend
y testear los eventos en tiempo real, **separados por rol**.

---

## 1) Endpoint WebSocket

- URL local: `ws://127.0.0.1:8080/ws`
- Método de conexión: WebSocket estándar.

### Conexión por rol

El rol se pasa por el parámetro de query `?role=` en la URL del WebSocket.

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

> **Valores válidos para `?role=`:** `admin` | `supervisor` | `cliente` | `cajero`
>
> Si no pasas el parámetro, el rol por defecto es `supervisor`.
>
> Podés pegar todo este código en la **consola del navegador** (F12 → Console)
> para conectar los 4 roles al mismo tiempo.

---

## 2) Formato general de mensajes

Todos los mensajes usan el esquema:

```json
{
  "type": "EVENT_NAME",
  "payload": { }
}
```

---

## 3) Eventos por rol

### 3.1 Supervisor

#### `WASH_STATUS_UPDATE`
Notifica cambio de estado de lavado detectado en la tabla `sales`.

```json
{
  "sale_id": 123,
  "placa": "Vehículo #1",
  "nuevo_estado": "En proceso"
}
```

#### `SUPERVISOR_SALES_SUMMARY`
Resumen agregado por estado de lavado.

```json
{
  "pendientes": 4,
  "en_proceso": 2,
  "completadas": 1,
  "canceladas": 0
}
```

Mapeo:
| Campo         | Estado `washing_status` |
|---------------|------------------------|
| `pendientes`  | `W` — En espera        |
| `en_proceso`  | `I` — En proceso       |
| `completadas` | `D` — Completado       |
| `canceladas`  | `C` — Cancelado        |

#### `SUPERVISOR_SALE_STATE_CHANGED`
Cambio de estado de una venta entre ciclos de monitoreo.

```json
{
  "sale_id": 123,
  "estado_anterior": "En espera",
  "estado_actual": "En proceso"
}
```

#### `NEW_SALE_CREATED`
Nueva venta registrada en el sistema.

```json
{
  "sale_id": 123,
  "tipo_vehiculo": "Vehículo",
  "servicios": ["Lavado"]
}
```

#### `UNPAID_SALES_ALERT` *(nuevo)*
Alerta de ventas sin pagar, emitida junto con el resumen de ventas.

```json
{
  "cantidad_sin_pagar": 2,
  "monto_total_pendiente": 0.0
}
```

#### `EXPIRED_COMBOS_ALERT` *(nuevo)*
Alerta de combos o promociones que ya vencieron.

```json
{
  "cantidad_vencidos": 1,
  "combos": ["Combo Básico Vencido"]
}
```

#### `EXPIRING_COMBOS_ALERT` *(nuevo)*
Alerta de combos o promociones próximos a vencer (ej: en los próximos 3 días).

```json
{
  "cantidad_por_vencer": 1,
  "dias_restantes": 3,
  "combos": ["Combo Premium"]
}
```

#### `NEW_ASSIGNMENT`
Nueva asignación de lavado a un empleado.

```json
{
  "nombre_servicio": "Lavado básico",
  "placa": "ABC-123"
}
```

---

### 3.2 Cajero

#### `NEW_SALE_CREATED`
(Compartido con Supervisor) Nueva venta registrada.

```json
{
  "sale_id": 123,
  "tipo_vehiculo": "Vehículo",
  "servicios": ["Lavado"]
}
```

#### `UNPAID_SALES_ALERT` *(nuevo)*
Alerta de ventas sin pagar, recibida junto con el resumen de cobros.

```json
{
  "cantidad_sin_pagar": 2,
  "monto_total_pendiente": 0.0
}
```

#### `PENDING_PAYMENT_REMINDER` *(nuevo)*
Recordatorio de pago pendiente para el Cajero.

```json
{
  "sale_id": 123,
  "nombre_cliente": "Cliente #1",
  "monto_pendiente": 0.0,
  "dias_pendiente": 0
}
```

#### `CASHIER_DAILY_SUMMARY` *(nuevo)*
Resumen de cobros del día para el Cajero.

```json
{
  "total_recaudado": 0.0,
  "cantidad_pendientes": 1,
  "cantidad_transacciones": 5
}
```

#### `PAYMENT_RECEIVED`
Pago confirmado / factura generada.

```json
{
  "sale_id": 123,
  "nombre_cliente": "Juan Pérez",
  "monto": 25.0,
  "metodo_pago": "Efectivo"
}
```

---

### 3.3 Cliente

#### `VEHICLE_PENDING` *(nuevo)*
El vehículo del cliente está pendiente / en espera de lavado.

```json
{
  "placa": "ABC-123",
  "nombre_servicio": "Lavado"
}
```

#### `VEHICLE_IN_PROGRESS`
El vehículo del cliente entró en proceso de lavado.

```json
{
  "placa": "ABC-123",
  "nombre_servicio": "Lavado en proceso"
}
```

#### `VEHICLE_READY`
El vehículo del cliente está listo / lavado terminado.

```json
{
  "placa": "ABC-123",
  "nombre_servicio": "Lavado completado"
}
```

#### `VEHICLE_CANCELED` *(nuevo)*
El lavado del vehículo del cliente fue cancelado.

```json
{
  "placa": "ABC-123",
  "nombre_servicio": "Lavado",
  "motivo": "Cancelado por el supervisor"
}
```

#### `ASSIGNED_LAVADOR`
Se asignó un lavador al vehículo del cliente.

```json
{
  "nombre_empleado": "Carlos López",
  "nombre_servicio": "Lavado básico"
}
```

---

### 3.4 Admin

#### `STOCK_ALERT`
Alerta de stock crítico en inventario.

```json
{
  "nombre_producto": "Shampoo",
  "stock_actual": 2.0,
  "stock_minimo": 5.0
}
```

#### `EXPIRED_COMBOS_ALERT` *(nuevo)*
Alerta de combos o promociones que ya vencieron.

```json
{
  "cantidad_vencidos": 1,
  "combos": ["Combo Básico Vencido"]
}
```

#### `EXPIRING_COMBOS_ALERT` *(nuevo)*
Alerta de combos o promociones próximos a vencer.

```json
{
  "cantidad_por_vencer": 1,
  "dias_restantes": 3,
  "combos": ["Combo Premium"]
}
```

#### `URGENT_PURCHASE`
Factura pendiente de pago a proveedor.

```json
{
  "nombre_proveedor": "Distribuidora XYZ",
  "numero_factura": "FAC-00123",
  "monto_pendiente": 1500.0
}
```

---

### 3.5 Eventos compartidos / utilidad

| Evento                    | Descripción                    | Roles que lo reciben              |
|---------------------------|--------------------------------|-----------------------------------|
| `WASH_STATUS_UPDATE`      | Cambio de estado general       | Supervisor, Cajero                |
| `NEW_SALE_CREATED`        | Nueva venta en el sistema      | Supervisor, Cajero                |
| `UNPAID_SALES_ALERT`      | Ventas sin pagar               | Supervisor, Cajero                |
| `VEHICLE_PENDING`         | Vehículo en espera de lavado   | Cliente                           |
| `PING`                    | Heartbeat / conexión viva      | Todos                             |

---

## 4) Estrategia recomendada en frontend

1. **Crear un dispatcher por `type`** (ver ejemplo abajo).
2. **Validar `payload`** antes de pintar UI.
3. **Mantener un store de ventas** por `sale_id`.
4. Aplicar `SUPERVISOR_SALE_STATE_CHANGED` de forma incremental.
5. Usar `SUPERVISOR_SALES_SUMMARY` para dashboards/contadores globales.
6. **Filtrar eventos por rol** en el frontend como capa adicional de seguridad.

Ejemplo de dispatcher:

```js
function handleWsMessage(raw, userRole) {
  const message = JSON.parse(raw);

  switch (message.type) {
    // ── Supervisor ──────────────────────────────────────────
    case 'WASH_STATUS_UPDATE':
      // actualizar tarjeta de venta
      break;
    case 'SUPERVISOR_SALES_SUMMARY':
      // actualizar KPIs del tablero del supervisor
      break;
    case 'SUPERVISOR_SALE_STATE_CHANGED':
      // actualizar estado puntual por sale_id
      break;
    case 'NEW_SALE_CREATED':
      // agregar venta al listado (Supervisor y Cajero)
      break;
    case 'NEW_ASSIGNMENT':
      // mostrar nueva asignación al empleado
      break;

    // ── Cajero ──────────────────────────────────────────────
    case 'PENDING_PAYMENT_REMINDER':
      // mostrar alerta de pago pendiente en la caja
      break;
    case 'CASHIER_DAILY_SUMMARY':
      // actualizar resumen de cobros del día
      break;
    case 'PAYMENT_RECEIVED':
      // confirmar pago recibido, actualizar saldo
      break;

    // ── Cliente ─────────────────────────────────────────────
    case 'VEHICLE_IN_PROGRESS':
      // notificar al cliente: su vehículo está siendo lavado
      break;
    case 'VEHICLE_READY':
      // notificar al cliente: su vehículo está listo
      break;
    case 'ASSIGNED_LAVADOR':
      // mostrar quién está lavando su vehículo
      break;

    // ── Admin ───────────────────────────────────────────────
    case 'STOCK_ALERT':
      // mostrar alerta de inventario bajo
      break;
    case 'URGENT_PURCHASE':
      // mostrar factura pendiente de pago a proveedor
      break;

    // ── Utilidad ────────────────────────────────────────────
    case 'PING':
      // heartbeat opcional
      break;

    default:
      console.warn('Tipo WS no manejado:', message.type);
  }
}
```

---

## 5) Checklist de testing por rol

### Supervisor
- [ ] Conectar a `ws://127.0.0.1:8080/ws` sin errores.
- [ ] Recibir `WASH_STATUS_UPDATE` tras insertar semilla.
- [ ] Recibir `SUPERVISOR_SALES_SUMMARY` inicial.
- [ ] Recibir `SUPERVISOR_SALE_STATE_CHANGED` tras transición de estados.
- [ ] Recibir `NEW_SALE_CREATED` por cada venta nueva.

### Cajero
- [ ] Conectar sin errores.
- [ ] Recibir `NEW_SALE_CREATED` por cada venta de la semilla.
- [ ] Recibir `PENDING_PAYMENT_REMINDER` por ventas sin pagar (`statussale = W`).
- [ ] Recibir `CASHIER_DAILY_SUMMARY` con conteos correctos.
- [ ] **NO** recibe `SUPERVISOR_SALES_SUMMARY` ni `VEHICLE_READY`.

### Cliente
- [ ] Conectar sin errores.
- [ ] Recibir `VEHICLE_IN_PROGRESS` al cambiar a `En proceso`.
- [ ] Recibir `VEHICLE_READY` al cambiar a `Completado`.
- [ ] **NO** recibe eventos de Supervisor ni Cajero.

### Admin
- [ ] Conectar sin errores.
- [ ] Recibir `WASH_STATUS_UPDATE` y `NEW_SALE_CREATED`.
- [ ] **NO** recibe `SUPERVISOR_SALES_SUMMARY` ni eventos de Cliente.

---

## 6) Frecuencia de monitoreo backend

El backend consulta periódicamente la DB cada **5 segundos**. Por esto, las
notificaciones de estado/resumen llegan en ventanas aproximadas de ese intervalo.

## 7) Consideraciones importantes para el frontend

### Conexiones WebSocket y reinicios del servidor
Al reiniciar el backend (`cargo run`), las conexiones WebSocket del lado del cliente
se quedan huérfanas aunque la pestaña del navegador siga pareciendo conectada.
Es necesario reconectar manualmente recargando la página o estableciendo una nueva
conexión WebSocket después de que el backend haya vuelto a estar disponible.

### Rol Admin y sus notificaciones
El rol Admin **solo recibe notificaciones de la capa estratégica**, como:
- `STOCK_ALERT` (alertas de inventario crítico)
- `EXPIRED_COMBOS_ALERT` y `EXPIRING_COMBOS_ALERT` (gestión de promociones)
- `URGENT_PURCHASE` (facturas pendientes de pago a proveedores)

El Admin **no recibe notificaciones operativas de ventas** como `WASH_STATUS_UPDATE`,
`SUPERVISOR_SALES_SUMMARY` o `NEW_SALE_CREATED`, ya que estas están reservadas
para los roles de Supervisor y Cajero.

## 7) Consideraciones importantes para el frontend

### Conexiones WebSocket y reinicios del servidor
Al reiniciar el backend (`cargo run`), las conexiones WebSocket del lado del cliente
se quedan huérfanas aunque la pestaña del navegador siga pareciendo conectada.
Es necesario reconectar manualmente recargando la página o estableciendo una nueva
conexión WebSocket después de que el backend haya vuelto a estar disponible.

### Rol Admin y sus notificaciones
El rol Admin **solo recibe notificaciones de la capa estratégica**, como:
- `STOCK_ALERT` (alertas de inventario crítico)
- `EXPIRED_COMBOS_ALERT` y `EXPIRING_COMBOS_ALERT` (gestión de promociones)
- `URGENT_PURCHASE` (facturas pendientes de pago a proveedores)

El Admin **no recibe notificaciones operativas de ventas** como `WASH_STATUS_UPDATE`,
`SUPERVISOR_SALES_SUMMARY` o `NEW_SALE_CREATED`, ya que estas están reservadas
para los roles de Supervisor y Cajero.

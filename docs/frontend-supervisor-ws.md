# API Actix Web - Guía Frontend para notificaciones de Supervisor

Este documento describe cómo el frontend puede conectarse al WebSocket del backend y testear los eventos en tiempo real relacionados con las tablas usadas por el supervisor (especialmente `sales`).

## 1) Endpoint WebSocket

- URL local: `ws://127.0.0.1:8080/ws`
- Método de conexión: WebSocket estándar.

Ejemplo rápido en JavaScript:

```js
const ws = new WebSocket('ws://127.0.0.1:8080/ws');

ws.onopen = () => {
  console.log('WS conectado');
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Evento WS:', data);
};

ws.onerror = (err) => {
  console.error('Error WS:', err);
};

ws.onclose = () => {
  console.log('WS desconectado');
};
```

## 2) Formato general de mensajes

Todos los mensajes usan el esquema:

```json
{
  "type": "EVENT_NAME",
  "payload": { }
}
```

## 3) Eventos actuales para supervisor

### A. `WASH_STATUS_UPDATE`
Notifica lavados terminados detectados en la tabla `sales`.

Payload:

```json
{
  "sale_id": 123,
  "plate": "Vehículo",
  "new_status": "Terminado"
}
```

### B. `SUPERVISOR_SALES_SUMMARY`
Resumen agregado por estado de lavado para la tabla `sales`.

Payload:

```json
{
  "pending": 4,
  "in_progress": 2,
  "finished": 5,
  "delivered": 1,
  "canceled": 0
}
```

Mapeo esperado:
- `pending` => estado `En espera`
- `in_progress` => estado `En proceso`
- `finished` => estado `Terminado`
- `delivered` => estado `Entregado`
- `canceled` => estado `Cancelado`

### C. `SUPERVISOR_SALE_STATE_CHANGED`
Notifica cuando una venta cambia de estado entre ciclos de monitoreo.

Payload:

```json
{
  "sale_id": 123,
  "previous_status": "En espera",
  "current_status": "En proceso"
}
```

### D. `NEW_SALE_CREATED`
Evento disponible en el contrato de mensajes para nuevas ventas.

Payload:

```json
{
  "sale_id": 123,
  "vehicle_type": "Sedan",
  "services": ["Lavado básico", "Encerado"]
}
```

### E. `STOCK_ALERT`
Evento disponible en el contrato de mensajes para alertas de inventario.

Payload:

```json
{
  "product_name": "Shampoo",
  "current_stock": 2.0,
  "min_stock": 5.0
}
```

### F. `PING`
Evento simple para validación de conexión.

Payload: sin campos adicionales.

## 4) Estrategia recomendada en frontend

1. Crear un dispatcher por `type`.
2. Validar `payload` antes de pintar UI.
3. Mantener un store de ventas por `sale_id`.
4. Aplicar `SUPERVISOR_SALE_STATE_CHANGED` de forma incremental.
5. Usar `SUPERVISOR_SALES_SUMMARY` para dashboards/contadores globales.

Ejemplo de dispatcher:

```js
function handleWsMessage(raw) {
  const message = JSON.parse(raw);

  switch (message.type) {
    case 'WASH_STATUS_UPDATE':
      // actualizar tarjeta de venta finalizada
      break;
    case 'SUPERVISOR_SALES_SUMMARY':
      // actualizar KPIs del tablero
      break;
    case 'SUPERVISOR_SALE_STATE_CHANGED':
      // actualizar estado puntual por sale_id
      break;
    case 'NEW_SALE_CREATED':
      // agregar venta al listado
      break;
    case 'STOCK_ALERT':
      // mostrar alerta visual/sonora
      break;
    case 'PING':
      // heartbeat opcional
      break;
    default:
      console.warn('Tipo WS no manejado:', message.type);
  }
}
```

## 5) Checklist de testing para frontend

- Conectar a `ws://127.0.0.1:8080/ws` sin errores.
- Verificar parseo JSON de todos los mensajes.
- Simular cambios de estado en `sales` y confirmar recepción de:
  - `SUPERVISOR_SALE_STATE_CHANGED`
  - `SUPERVISOR_SALES_SUMMARY`
- Confirmar actualización visual de dashboard y listado.
- Validar reconexión automática de WebSocket al caer conexión.

## 6) Frecuencia de monitoreo backend

El backend consulta periódicamente la DB cada 5 segundos. Por esto, las notificaciones de estado/resumen llegan en ventanas aproximadas de ese intervalo.

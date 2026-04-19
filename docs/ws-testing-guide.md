# Guía completa de pruebas WS (Supervisor)

Esta guía cubre el flujo completo de pruebas para notificaciones en tiempo real del supervisor: preparación, ejecución, resultados esperados, troubleshooting y limpieza.

---

## 1) Objetivo

Validar que el backend:

1. **No cambia estructura de tablas** (sin DDL).
2. Usa lectura de `sales` y emite eventos WS correctos.
3. Reacciona a cambios de `stateuswashing` con notificaciones en tiempo real.

---

## 2) Eventos WS a validar

### `WASH_STATUS_UPDATE`
- Se emite por cada `sale_id` detectado por primera vez en memoria.
- Estado enviado: valor real de `stateuswashing`.

Ejemplo:

```json
{
  "type": "WASH_STATUS_UPDATE",
  "payload": {
    "sale_id": 90001,
    "plate": "Véhicule",
    "new_status": "En espera"
  }
}
```

### `SUPERVISOR_SALE_STATE_CHANGED`
- Se emite cuando una venta conocida cambia de estado entre ciclos de polling.

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

Ejemplo:

```json
{
  "type": "SUPERVISOR_SALES_SUMMARY",
  "payload": {
    "pending": 1,
    "in_progress": 2,
    "finished": 1,
    "delivered": 1,
    "canceled": 0
  }
}
```

---

## 3) Pre-requisitos

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

## 4) Scripts de prueba disponibles

- Semilla de datos: [`scripts/sql/ws_seed_sales_test.sql`](scripts/sql/ws_seed_sales_test.sql)
- Transiciones de estado: [`scripts/sql/ws_state_transition_test.sql`](scripts/sql/ws_state_transition_test.sql)

> Los scripts usan `saleid` 90001..90005 para no mezclarse con datos operativos.

---

## 5) Ejecución paso a paso

### Paso A: Conectar cliente WS

Puedes usar tu frontend o una consola de navegador:

```js
const ws = new WebSocket('ws://127.0.0.1:8080/ws');
ws.onmessage = (e) => console.log(JSON.parse(e.data));
```

### Paso B: Insertar semilla

Ejecuta en PostgreSQL:

```sql
\i scripts/sql/ws_seed_sales_test.sql
```

Espera entre 5 y 10 segundos (polling del backend).

### Paso C: Verificar eventos iniciales

Debes observar:

1. Varios `WASH_STATUS_UPDATE` para `sale_id` nuevos.
2. Un `SUPERVISOR_SALES_SUMMARY` inicial con conteos de la semilla.

### Paso D: Forzar transición de estados

Ejecuta:

```sql
\i scripts/sql/ws_state_transition_test.sql
```

Espera 5-10 segundos.

### Paso E: Verificar eventos de cambio

Debes observar:

1. `SUPERVISOR_SALE_STATE_CHANGED` por cada venta afectada.
2. `SUPERVISOR_SALES_SUMMARY` actualizado.

---

## 6) Matriz rápida (entrada -> salida esperada)

| Acción en DB | Evento esperado | Validación |
|---|---|---|
| Insertar `saleid=90001` con `En espera` | `WASH_STATUS_UPDATE` | `payload.new_status = "En espera"` |
| Cambiar 90001 a `En proceso` | `SUPERVISOR_SALE_STATE_CHANGED` | `previous_status = "En espera"`, `current_status = "En proceso"` |
| Cambiar varias ventas | `SUPERVISOR_SALES_SUMMARY` | Conteos coinciden con `SELECT ... GROUP BY stateuswashing` |

---

## 7) Consultas SQL de verificación manual

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

## 8) Limpieza de datos de prueba

Cuando termines:

```sql
DELETE FROM sales
WHERE saleid BETWEEN 90001 AND 90005;
```

---

## 9) Troubleshooting

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

---

## 10) Checklist final

- [ ] Se conectó WS sin errores.
- [ ] Se recibieron `WASH_STATUS_UPDATE` tras semilla.
- [ ] Se recibió `SUPERVISOR_SALES_SUMMARY` inicial.
- [ ] Se recibieron `SUPERVISOR_SALE_STATE_CHANGED` tras transición.
- [ ] El resumen final coincide con SQL.
- [ ] Se limpiaron datos de prueba (90001..90005).

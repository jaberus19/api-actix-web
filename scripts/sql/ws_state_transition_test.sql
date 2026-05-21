-- ============================================================
-- Transiciones de estado para validar eventos WS incrementales
-- Roles: Admin | Supervisor | Cliente | Cajero
-- ============================================================
-- Ejecutar después de ws_seed_sales_test.sql y con backend corriendo.
-- Los valores de enum deben coincidir con los definidos en la base de datos:
--   washing_status: W=En espera, I=En proceso, D=Completado, C=Cancelado
--   status_payments: W=Waiting, P=Paid, C=Cancelled
--
-- Eventos esperados por transición:
--   90001 W→I  → Supervisor: SUPERVISOR_SALE_STATE_CHANGED
--              → Cliente:     VEHICLE_IN_PROGRESS
--   90002 W→D  → Supervisor: SUPERVISOR_SALE_STATE_CHANGED
--              → Cliente:     VEHICLE_READY
--   90003 D→C  → Supervisor: SUPERVISOR_SALE_STATE_CHANGED
--              → Cliente:     VEHICLE_CANCELED
--   90004 C→C  → sin cambios (mismo estado, no hay evento)
--   statussale W→P (90001) → Cajero: pago confirmado
--
BEGIN;

-- ── Transiciones de estado de lavado ─────────────────────────

UPDATE sales
SET "statusWashing" = 'I'
WHERE "saleId" = 90001;  -- De 'W' (En espera) a 'I' (En proceso)
                         -- → Supervisor: SUPERVISOR_SALE_STATE_CHANGED
                         -- → Cliente:     VEHICLE_IN_PROGRESS

UPDATE sales
SET "statusWashing" = 'D'
WHERE "saleId" = 90002;  -- De 'W' (En espera) a 'D' (Completado)
                         -- → Supervisor: SUPERVISOR_SALE_STATE_CHANGED
                         -- → Cliente:     VEHICLE_READY

UPDATE sales
SET "statusWashing" = 'C'
WHERE "saleId" = 90003;  -- De 'D' (Completado) a 'C' (Cancelado)
                         -- → Supervisor: SUPERVISOR_SALE_STATE_CHANGED
                         -- → Cliente:     VEHICLE_CANCELED

UPDATE sales
SET "statusWashing" = 'C'
WHERE "saleId" = 90004;  -- Ya estaba en 'C', se mantiene (sin evento)

-- ── Simulación de pago recibido por Cajero ───────────────────
-- Cambia statussale de 'W' (Waiting) a 'P' (Paid)
-- → Cajero: deja de recibir PENDING_PAYMENT_REMINDER para esta venta
UPDATE sales
SET "statusSale" = 'P'
WHERE "saleId" = 90001;   -- Pago confirmado por Cajero

COMMIT;

-- Verificación rápida
SELECT "saleId", "statusWashing", "statusSale", "saleDate", "invoiceNumber"
FROM sales
WHERE "saleId" BETWEEN 90001 AND 90005
ORDER BY "saleId";

-- ============================================================
-- Eventos esperados después de ejecutar este script:
-- ============================================================
--   Supervisor  → SUPERVISOR_SALE_STATE_CHANGED × 3
--               → SUPERVISOR_SALES_SUMMARY actualizado
--   Cajero      → CASHIER_DAILY_SUMMARY actualizado
--               → UNPAID_SALES_ALERT actualizado
--   Cliente     → VEHICLE_IN_PROGRESS (90001)
--               → VEHICLE_READY (90002)
--               → VEHICLE_CANCELED (90003)
--   Admin       → WASH_STATUS_UPDATE por cada cambio

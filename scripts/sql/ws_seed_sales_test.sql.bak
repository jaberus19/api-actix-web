-- ============================================================
-- Semilla Integral de datos para pruebas de notificaciones WS
-- Corregido error de sintaxis en ON CONFLICT
-- ============================================================

BEGIN;

-- 1. Insertar Tipo de Vehículo base
INSERT INTO types_vehicles ("typeVehicleId", "name")
VALUES (1, 'Sedán')
ON CONFLICT ("typeVehicleId") DO NOTHING;

-- 2. Insertar Método de Pago base (Sintaxis corregida para PostgreSQL)
INSERT INTO payments_methods ("paymentMethodId", "name")
VALUES (1, 'Efectivo')
ON CONFLICT ("paymentMethodId") DO UPDATE 
SET name = EXCLUDED.name; 
-- Nota: Si 'Efectivo' ya existía en otro ID, esto podría fallar por el UNIQUE de name.
-- Si eso pasa, simplemente usamos una consulta limpia antes, pero probemos este enfoque directo primero.

-- 3. Insertar Cliente base
INSERT INTO clients ("clientId", "names", "lastnames", "numberPhone", "ci")
VALUES (1, 'Juan', 'Pérez', '0412-5555555', 'V-12345678')
ON CONFLICT ("clientId") DO NOTHING;

-- 4. Insertar Vehículo base
INSERT INTO vehicles ("vehicleId", "typeVehicleId", "ownerId", "plate")
VALUES (1, 1, 1, 'ABC123X')
ON CONFLICT ("vehicleId") DO NOTHING;

-- 5. Insertar Ventas de prueba mapeadas a CamelCase exacto
INSERT INTO sales (
  "saleId",
  "clientId",
  "vehicleId",
  "paymentMethodId",
  "statusSale",
  "statusWashing",
  "saleDate",
  "initialState"
)
VALUES
  (90001, 1, 1, 1, 'W', 'W', NOW(), 'En espera'),  
  (90002, 1, 1, 1, 'W', 'I', NOW(), 'En espera'),  
  (90003, 1, 1, 1, 'P', 'D', NOW(), 'En espera'),  
  (90004, 1, 1, 1, 'P', 'C', NOW(), 'En espera'),  
  (90005, 1, 1, 1, 'C', 'C', NOW(), 'En espera')   
ON CONFLICT ("saleId") DO UPDATE
SET
  "statusWashing" = EXCLUDED."statusWashing",
  "statusSale"    = EXCLUDED."statusSale",
  "saleDate"      = EXCLUDED."saleDate",
  "initialState"  = EXCLUDED."initialState";

COMMIT;

-- Verificación de salida
SELECT "saleId", "statusWashing", "statusSale", "saleDate"
FROM sales
WHERE "saleId" BETWEEN 90001 AND 90005
ORDER BY "saleId";
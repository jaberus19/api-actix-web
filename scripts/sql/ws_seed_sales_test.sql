-- ============================================================
-- Semilla Integral de datos para pruebas de notificaciones WS
-- Corregido error de sintaxis en ON CONFLICT
-- ============================================================

BEGIN;

-- 0. Asegurar que la tabla sales exista
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'sales') THEN
        CREATE TABLE sales (
            "saleId" SERIAL PRIMARY KEY,
            "clientId" INTEGER NOT NULL,
            "vehicleId" INTEGER NOT NULL,
            "paymentMethodId" INTEGER NOT NULL,
            "statusSale" TEXT NOT NULL,
            "statusWashing" TEXT NOT NULL,
            "saleDate" TIMESTAMP NOT NULL,
            "initialState" TEXT,
            "invoiceNumber" VARCHAR
        );
    END IF;
END $$;

-- 0. Asegurar que la tabla sales tenga la columna invoiceNumber
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name='sales' AND column_name='invoiceNumber') THEN
        ALTER TABLE sales ADD COLUMN "invoiceNumber" VARCHAR;
    END IF;
END $$;

-- Actualizar filas existentes con invoiceNumber nulo (usando un valor temporal basado en saleId)
UPDATE sales SET "invoiceNumber" = 'N-A-' || "saleId" WHERE "invoiceNumber" IS NULL;

-- Aplicar restricciones NOT NULL y UNIQUE a invoiceNumber
ALTER TABLE sales ALTER COLUMN "invoiceNumber" SET NOT NULL;
ALTER TABLE sales ADD CONSTRAINT sales_invoicenumber_unique UNIQUE ("invoiceNumber");

-- 1. Crear tabla products si no existe (necesaria para product_usage)
CREATE TABLE IF NOT EXISTS products (
    "productId" SERIAL PRIMARY KEY,
    "name" VARCHAR NOT NULL
);

-- 1. Crear tabla product_usage si no existe (según informe de auditoría)
CREATE TABLE IF NOT EXISTS product_usage (
    "productUsageId" SERIAL PRIMARY KEY,
    "productId" INTEGER NOT NULL REFERENCES products("productId") ON DELETE CASCADE,
    "quantityUsed" NUMERIC(10, 2) NOT NULL,
    "unitType" TEXT NOT NULL DEFAULT 'L',
    "createdAt" TIMESTAMP DEFAULT NOW ()
);

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

-- 5. Insertar Ventas de prueba mapeadas a los nombres exactos de columna (CamelCase)
INSERT INTO sales (
   "saleId",
   "clientId",
   "vehicleId",
   "paymentMethodId",
   "statusSale",
   "statusWashing",
   "saleDate",
   "initialState",
   "invoiceNumber"
)
VALUES
   (90001, 1, 1, 1, 'W', 'W', NOW(), 'En espera', 'N-A-90001'),  
   (90002, 1, 1, 1, 'W', 'I', NOW(), 'En espera', 'N-A-90002'),  
   (90003, 1, 1, 1, 'P', 'D', NOW(), 'En espera', 'N-A-90003'),  
   (90004, 1, 1, 1, 'P', 'C', NOW(), 'En espera', 'N-A-90004'),  
   (90005, 1, 1, 1, 'C', 'C', NOW(), 'En espera', 'N-A-90005')   
ON CONFLICT ("saleId") DO UPDATE
SET
   "statusWashing" = EXCLUDED."statusWashing",
   "statusSale"    = EXCLUDED."statusSale",
   "saleDate"      = EXCLUDED."saleDate",
   "initialState"  = EXCLUDED."initialState",
   "invoiceNumber" = EXCLUDED."invoiceNumber";

COMMIT;

-- Verificación de salida
SELECT "saleId", "statusWashing", "statusSale", "saleDate", "invoiceNumber"
FROM sales
WHERE "saleId" BETWEEN 90001 AND 90005
ORDER BY "saleId";
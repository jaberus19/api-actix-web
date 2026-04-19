-- Transiciones de estado para validar eventos WS incrementales
-- Ejecutar después de ws_seed_sales_test.sql y con backend corriendo.

BEGIN;

UPDATE sales
SET stateuswashing = 'En proceso'
WHERE saleid = 90001;

UPDATE sales
SET stateuswashing = 'Terminado'
WHERE saleid = 90002;

UPDATE sales
SET stateuswashing = 'Entregado'
WHERE saleid = 90003;

UPDATE sales
SET stateuswashing = 'Cancelado'
WHERE saleid = 90004;

COMMIT;

-- Verificación rápida
SELECT saleid, stateuswashing, statussale, saledate
FROM sales
WHERE saleid BETWEEN 90001 AND 90005
ORDER BY saleid;


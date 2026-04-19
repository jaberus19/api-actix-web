-- Semilla de datos para probar notificaciones WS de supervisor
-- Inserta ventas de prueba en distintos estados de `stateuswashing`.
-- Nota: ajusta IDs de FK si en tu entorno no existen estos valores.

BEGIN;

INSERT INTO sales (
  saleid,
  clientid,
  vehicleid,
  paymentmethodid,
  statussale,
  stateuswashing,
  saledate,
  initial_state
)
VALUES
  (90001, 1, 1, 1, 'Pendiente', 'En espera', NOW(), 'En espera'),
  (90002, 1, 1, 1, 'Pendiente', 'En proceso', NOW(), 'En espera'),
  (90003, 1, 1, 1, 'Pagado',    'Terminado', NOW(), 'En espera'),
  (90004, 1, 1, 1, 'Pagado',    'Entregado', NOW(), 'En espera'),
  (90005, 1, 1, 1, 'Anulado',   'Cancelado', NOW(), 'En espera')
ON CONFLICT (saleid) DO UPDATE
SET
  stateuswashing = EXCLUDED.stateuswashing,
  statussale = EXCLUDED.statussale,
  saledate = EXCLUDED.saledate,
  initial_state = EXCLUDED.initial_state;

COMMIT;

-- Verificación rápida
SELECT saleid, stateuswashing, statussale, saledate
FROM sales
WHERE saleid BETWEEN 90001 AND 90005
ORDER BY saleid;


-- Seed data for fuel_prices table
-- Pastikan data SPBU sudah ada sebelum menjalankan seed ini

-- Harga bahan bakar untuk SPBU 1
INSERT INTO fuel_prices (spbu_id, fuel_type, price)
SELECT id, 'PERTAMAX', 12500.00 FROM spbu LIMIT 1
ON CONFLICT (spbu_id, fuel_type) DO UPDATE 
SET price = EXCLUDED.price, updated_at = NOW();

-- Harga bahan bakar untuk SPBU 1
INSERT INTO fuel_prices (spbu_id, fuel_type, price)
SELECT id, 'PERTALITE', 10000.00 FROM spbu LIMIT 1
ON CONFLICT (spbu_id, fuel_type) DO UPDATE 
SET price = EXCLUDED.price, updated_at = NOW();

-- Harga bahan bakar untuk SPBU 1
INSERT INTO fuel_prices (spbu_id, fuel_type, price)
SELECT id, 'SOLAR', 8500.00 FROM spbu LIMIT 1
ON CONFLICT (spbu_id, fuel_type) DO UPDATE 
SET price = EXCLUDED.price, updated_at = NOW();

-- Harga bahan bakar untuk SPBU 1
INSERT INTO fuel_prices (spbu_id, fuel_type, price)
SELECT id, 'PERTAMAX TURBO', 13500.00 FROM spbu LIMIT 1
ON CONFLICT (spbu_id, fuel_type) DO UPDATE 
SET price = EXCLUDED.price, updated_at = NOW();

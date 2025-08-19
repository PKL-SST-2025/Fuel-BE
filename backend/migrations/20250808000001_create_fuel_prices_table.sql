-- Create fuel_prices table
CREATE TABLE IF NOT EXISTS fuel_prices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    spbu_id UUID NOT NULL,
    fuel_type VARCHAR(50) NOT NULL,
    price DECIMAL(12, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Add foreign key constraint for spbu_id
    CONSTRAINT fk_spbu FOREIGN KEY(spbu_id) REFERENCES spbu(id) ON DELETE CASCADE,
    
    -- Ensure unique combination of spbu_id and fuel_type
    CONSTRAINT uq_spbu_fuel_type UNIQUE(spbu_id, fuel_type)
);

-- Create index for faster lookups
CREATE INDEX idx_fuel_prices_spbu_id ON fuel_prices(spbu_id);
CREATE INDEX idx_fuel_prices_fuel_type ON fuel_prices(fuel_type);

-- Add trigger for updating updated_at
CREATE OR REPLACE FUNCTION update_fuel_prices_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_fuel_prices_updated_at
BEFORE UPDATE ON fuel_prices
FOR EACH ROW
EXECUTE FUNCTION update_fuel_prices_updated_at();

-- Add comments for better documentation
COMMENT ON TABLE fuel_prices IS 'Stores fuel prices for each SPBU and fuel type';
COMMENT ON COLUMN fuel_prices.spbu_id IS 'Reference to the SPBU';
COMMENT ON COLUMN fuel_prices.fuel_type IS 'Type of fuel (e.g., PERTAMAX, PERTALITE, etc.)';
COMMENT ON COLUMN fuel_prices.price IS 'Price per liter in IDR';

-- Note: Data will be inserted after all migrations are complete

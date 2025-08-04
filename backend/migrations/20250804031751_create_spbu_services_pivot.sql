-- Create spbu_services pivot table
CREATE TABLE spbu_services (
    spbu_id UUID NOT NULL REFERENCES spbu(id) ON DELETE CASCADE,
    service_id UUID NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (spbu_id, service_id)
);

-- Create index for better query performance
CREATE INDEX idx_spbu_services_spbu_id ON spbu_services(spbu_id);
CREATE INDEX idx_spbu_services_service_id ON spbu_services(service_id);

-- Add updated_at trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_spbu_services_updated_at
BEFORE UPDATE ON spbu_services
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

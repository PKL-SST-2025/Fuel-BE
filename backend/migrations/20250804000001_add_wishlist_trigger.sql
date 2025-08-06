-- Buat function untuk update timestamp
CREATE OR REPLACE FUNCTION update_modified_column() 
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW; 
END;
$$ language 'plpgsql';

-- Tambahkan kolom updated_at ke tabel wishlists
ALTER TABLE wishlists 
ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Buat trigger
CREATE TRIGGER update_wishlists_modtime
BEFORE UPDATE ON wishlists
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();

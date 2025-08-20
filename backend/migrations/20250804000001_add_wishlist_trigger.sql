-- Hapus trigger dan function jika sudah ada
DROP TRIGGER IF EXISTS update_wishlists_modtime ON wishlists;
DROP FUNCTION IF EXISTS update_modified_column() CASCADE;

-- Buat function untuk update timestamp
CREATE OR REPLACE FUNCTION update_modified_column() 
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW; 
END;
$$ language 'plpgsql';

-- Tambahkan kolom updated_at ke tabel wishlists (jika belum ada)
ALTER TABLE IF EXISTS wishlists 
ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Buat trigger
CREATE OR REPLACE TRIGGER update_wishlists_modtime
BEFORE UPDATE ON wishlists
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();

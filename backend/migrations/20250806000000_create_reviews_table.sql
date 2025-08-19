-- Pastikan ekstensi uuid-ossp aktif
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Hapus trigger dan function yang mungkin sudah ada
DROP TRIGGER IF EXISTS update_spbu_rating_after_delete ON reviews;
DROP TRIGGER IF EXISTS update_spbu_rating_after_review ON reviews;
DROP TRIGGER IF EXISTS update_reviews_timestamp ON reviews;

-- Hapus function
DROP FUNCTION IF EXISTS update_spbu_rating();
DROP FUNCTION IF EXISTS update_review_timestamp();

-- Hapus index
DROP INDEX IF EXISTS idx_reviews_user_id;
DROP INDEX IF EXISTS idx_reviews_spbu_id;

-- Hapus tabel jika sudah ada
DROP TABLE IF EXISTS reviews CASCADE;

-- Buat tabel reviews
CREATE TABLE reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    spbu_id UUID NOT NULL REFERENCES spbu(id) ON DELETE CASCADE,
    rating FLOAT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, spbu_id)  -- Satu user hanya bisa memberikan 1 review per SPBU
);

-- Buat index untuk pencarian cepat
CREATE INDEX idx_reviews_user_id ON reviews(user_id);
CREATE INDEX idx_reviews_spbu_id ON reviews(spbu_id);

-- Buat function untuk update timestamp
CREATE OR REPLACE FUNCTION update_review_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Buat trigger
CREATE TRIGGER update_reviews_timestamp
BEFORE UPDATE ON reviews
FOR EACH ROW
EXECUTE FUNCTION update_review_timestamp();

-- Buat function untuk update rating rata-rata di tabel spbu
CREATE OR REPLACE FUNCTION update_spbu_rating()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE spbu
    SET rating = (
        SELECT COALESCE(AVG(rating), 0)
        FROM reviews
        WHERE spbu_id = COALESCE(NEW.spbu_id, OLD.spbu_id)
    )
    WHERE id = COALESCE(NEW.spbu_id, OLD.spbu_id);
    
    RETURN NULL;
END;
$$ language 'plpgsql';

-- Buat trigger untuk update rating setelah insert/update review
CREATE TRIGGER update_spbu_rating_after_review
AFTER INSERT OR UPDATE ON reviews
FOR EACH ROW
EXECUTE FUNCTION update_spbu_rating();

-- Buat trigger untuk update rating setelah delete review
CREATE TRIGGER update_spbu_rating_after_delete
AFTER DELETE ON reviews
FOR EACH ROW
EXECUTE FUNCTION update_spbu_rating();

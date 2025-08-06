-- Buat tabel wishlists
CREATE TABLE wishlists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    spbu_id UUID NOT NULL REFERENCES spbu(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, spbu_id)  -- Satu user hanya bisa wishlist satu SPBU sekali
);

-- Buat index untuk pencarian cepat
CREATE INDEX idx_wishlists_user_id ON wishlists(user_id);
CREATE INDEX idx_wishlists_spbu_id ON wishlists(spbu_id);

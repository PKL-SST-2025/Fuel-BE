CREATE TABLE spbu (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nama VARCHAR(100) NOT NULL,
    alamat TEXT NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    brand_id UUID REFERENCES brand(id),
    rating FLOAT DEFAULT 0,
    jumlah_pompa INT DEFAULT 0,
    jumlah_antrian INT DEFAULT 0,
    foto TEXT,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);
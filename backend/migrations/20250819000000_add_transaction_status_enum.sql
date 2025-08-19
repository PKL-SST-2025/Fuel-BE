-- Buat tipe enum untuk status transaksi
DO $$ 
BEGIN
    -- Cek apakah tipe enum sudah ada
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'transaction_status') THEN
        CREATE TYPE transaction_status AS ENUM ('pending', 'paid', 'processing', 'completed', 'cancelled');
    END IF;
END $$;

-- Hapus default constraint sebelum ubah tipe
ALTER TABLE transactions 
    ALTER COLUMN status DROP DEFAULT;

-- Ubah tipe kolom status
ALTER TABLE transactions 
    ALTER COLUMN status TYPE transaction_status 
    USING status::transaction_status;

-- Set default value setelah ubah tipe
ALTER TABLE transactions 
    ALTER COLUMN status SET DEFAULT 'pending';

-- Buat tipe enum untuk payment_status
DO $$ 
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'payment_status') THEN
        CREATE TYPE payment_status AS ENUM ('pending', 'paid', 'failed');
    END IF;
END $$;

-- Hapus default constraint sebelum ubah tipe
ALTER TABLE transactions 
    ALTER COLUMN payment_status DROP DEFAULT;

-- Ubah tipe kolom payment_status
ALTER TABLE transactions 
    ALTER COLUMN payment_status TYPE payment_status 
    USING payment_status::payment_status;

-- Set default value setelah ubah tipe
ALTER TABLE transactions 
    ALTER COLUMN payment_status SET DEFAULT 'pending';

-- Rôles utilisateurs
DO $$ BEGIN
    CREATE TYPE user_role AS ENUM ('admin', 'player');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- 1. Table des Utilisateurs
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role user_role DEFAULT 'player',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 2. Table des Serveurs FS25
CREATE TABLE IF NOT EXISTS servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    ip_address VARCHAR(45),
    port INT DEFAULT 10823,
    invite_code VARCHAR(20) UNIQUE NOT NULL,
    owner_id UUID REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 3. Table des Membres d'un Serveur (Many-to-Many)
CREATE TABLE IF NOT EXISTS server_members (
    server_id UUID REFERENCES servers(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (server_id, user_id)
);

-- 4. Catalogue Global des Mods Uploadés
CREATE TABLE IF NOT EXISTS mods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mod_name VARCHAR(100) NOT NULL,          -- Extrait de modDesc.xml (<title>)
    file_name VARCHAR(255) NOT NULL,         -- Nom original du .zip
    version VARCHAR(30) NOT NULL,           -- Extrait de modDesc.xml (<version>)
    hash_sha256 CHAR(64) NOT NULL UNIQUE,   -- Hash cryptographique SHA-256 pour vérification client
    file_size_bytes BIGINT NOT NULL,
    s3_key VARCHAR(512) NOT NULL,            -- Chemin de stockage dans le bucket S3
    uploaded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 5. Association Mods <-> Serveurs (Modpacks)
CREATE TABLE IF NOT EXISTS server_mods (
    server_id UUID REFERENCES servers(id) ON DELETE CASCADE,
    mod_id UUID REFERENCES mods(id) ON DELETE CASCADE,
    is_active BOOLEAN DEFAULT TRUE,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (server_id, mod_id)
);



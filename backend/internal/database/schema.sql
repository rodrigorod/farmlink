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

-- Index pour accélérer la recherche par hash (utilisé par l'app Rust)
CREATE INDEX IF NOT EXISTS idx_mods_hash ON mods(hash_sha256);

-- Données initiales de démonstration (Seed)
INSERT INTO users (id, username, email, password_hash, role)
VALUES (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    'Farmer_Alex',
    'alex@farmlink.io',
    '$2a$10$LoJzCrj4b8u2pbOsOWtXpuqrVzMSclBP6YD3O3ESpWO3C480YB6a.',
    'admin'
) ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash;

INSERT INTO servers (id, name, ip_address, port, invite_code, owner_id)
VALUES 
(
    'b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22',
    'Zielonka AgriPro Multi',
    'zielonka.farmlink-servers.eu',
    10823,
    'ZIELONKA2025',
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'
),
(
    'c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a33',
    'Riverbend Springs Co-op',
    'riverbend.farmlink-servers.eu',
    10824,
    'RIVERBEND2025',
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'
) ON CONFLICT (invite_code) DO NOTHING;

INSERT INTO mods (id, mod_name, file_name, version, hash_sha256, file_size_bytes, s3_key, uploaded_by)
VALUES
(
    'd0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44',
    'John Deere 7R Series 2024',
    'FS25_JohnDeere_7R.zip',
    '1.2.0.0',
    '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
    84500000,
    'mods/9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08/FS25_JohnDeere_7R.zip',
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'
),
(
    'e0eebc99-9c0b-4ef8-bb6d-6bb9bd380a55',
    'Courseplay FS25 NextGen',
    'FS25_Courseplay.zip',
    '7.4.1.0',
    'c83d6a9956d498b7921a24d550e0468e0d4948f93616238b1d4317135084922f',
    15400000,
    'mods/c83d6a9956d498b7921a24d550e0468e0d4948f93616238b1d4317135084922f/FS25_Courseplay.zip',
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'
),
(
    'f0eebc99-9c0b-4ef8-bb6d-6bb9bd380a66',
    'Claas Lexion 8900 Pack',
    'FS25_ClaasLexion8900.zip',
    '1.0.2.0',
    '03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4',
    132000000,
    'mods/03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4/FS25_ClaasLexion8900.zip',
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11'
) ON CONFLICT (hash_sha256) DO NOTHING;

INSERT INTO server_mods (server_id, mod_id, is_active)
VALUES
('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'd0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', true),
('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'e0eebc99-9c0b-4ef8-bb6d-6bb9bd380a55', true),
('b0eebc99-9c0b-4ef8-bb6d-6bb9bd380a22', 'f0eebc99-9c0b-4ef8-bb6d-6bb9bd380a66', true),
('c0eebc99-9c0b-4ef8-bb6d-6bb9bd380a33', 'd0eebc99-9c0b-4ef8-bb6d-6bb9bd380a44', true)
ON CONFLICT (server_id, mod_id) DO NOTHING;

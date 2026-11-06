-- ==============================================================================
-- Schema Inicial: Lista de Compras Colaborativa (PostgreSQL / Supabase)
-- Migração: 001_initial_schema.sql
-- ==============================================================================

-- 1. Extensões Necessárias
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- 2. Tabela de Categorias do Supermercado
CREATE TABLE IF NOT EXISTS categories (
    id VARCHAR(32) PRIMARY KEY,
    name TEXT NOT NULL,
    icon TEXT NOT NULL,
    color TEXT NOT NULL,
    sort_order INT NOT NULL DEFAULT 0
);

-- População das Categorias Padrão de Supermercado
INSERT INTO categories (id, name, icon, color, sort_order) VALUES
    ('produce', 'Hortifruti', 'apple', '#16a34a', 1),
    ('butcher', 'Açougue e Carnes', 'drumstick-bite', '#dc2626', 2),
    ('dairy', 'Laticínios e Frios', 'cheese', '#eab308', 3),
    ('bakery', 'Padaria e Confeitaria', 'croissant', '#d97706', 4),
    ('grocery', 'Mercearia e Grãos', 'wheat-awn', '#854d0e', 5),
    ('beverages', 'Bebidas', 'wine-bottle', '#0284c7', 6),
    ('cleaning', 'Limpeza', 'sparkles', '#0891b2', 7),
    ('hygiene', 'Higiene Pessoal', 'pump-soap', '#7c3aed', 8),
    ('frozen', 'Congelados', 'snowflake', '#2563eb', 9),
    ('other', 'Outros e Diversos', 'tags', '#64748b', 10)
ON CONFLICT (id) DO NOTHING;

-- 3. Tabela de Perfis de Usuários
-- Integrada ao Supabase Auth se existir auth.users, ou stand-alone para dev local
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    avatar_url TEXT,
    password_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 4. Tabela de Listas de Compras
CREATE TABLE IF NOT EXISTS lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    invite_code VARCHAR(16) NOT NULL UNIQUE,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 5. Membros da Lista (Vínculo de Usuários com Listas Compartilhadas)
CREATE TABLE IF NOT EXISTS list_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    list_id UUID NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member' CHECK (role IN ('owner', 'member')),
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_list_members UNIQUE (list_id, user_id)
);

-- 6. Itens da Lista de Compras
CREATE TABLE IF NOT EXISTS items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    list_id UUID NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    category_id VARCHAR(32) NOT NULL REFERENCES categories(id) ON UPDATE CASCADE,
    quantity NUMERIC(10, 2) NOT NULL DEFAULT 1.0 CHECK (quantity > 0),
    unit TEXT NOT NULL DEFAULT 'un',
    estimated_price NUMERIC(10, 2) CHECK (estimated_price >= 0),
    actual_price NUMERIC(10, 2) CHECK (actual_price >= 0),
    purchased BOOLEAN NOT NULL DEFAULT FALSE,
    purchased_at TIMESTAMPTZ,
    purchased_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    added_by_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 7. Histórico de Compras e Frequência para Sugestões Inteligentes
CREATE TABLE IF NOT EXISTS item_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    list_id UUID NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
    item_name_normalized TEXT NOT NULL,
    category_id VARCHAR(32) NOT NULL REFERENCES categories(id) ON UPDATE CASCADE,
    last_price NUMERIC(10, 2),
    last_unit TEXT NOT NULL DEFAULT 'un',
    purchase_count INT NOT NULL DEFAULT 1 CHECK (purchase_count >= 1),
    last_purchased_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_list_item_history UNIQUE (list_id, item_name_normalized)
);

-- ==============================================================================
-- 8. Índices de Otimização e Chaves Estrangeiras (Supabase / Postgres Best Practices)
-- ==============================================================================

-- Índices nas Chaves Estrangeiras de list_members
CREATE INDEX IF NOT EXISTS idx_list_members_list_id ON list_members(list_id);
CREATE INDEX IF NOT EXISTS idx_list_members_user_id ON list_members(user_id);

-- Índice Único e de Busca Rápida pelo Código de Convite da Lista
CREATE UNIQUE INDEX IF NOT EXISTS idx_lists_invite_code ON lists(UPPER(invite_code));
CREATE INDEX IF NOT EXISTS idx_lists_created_by ON lists(created_by);

-- Índices de Consulta de Itens da Lista
CREATE INDEX IF NOT EXISTS idx_items_list_id ON items(list_id);
CREATE INDEX IF NOT EXISTS idx_items_list_purchased ON items(list_id, purchased);
CREATE INDEX IF NOT EXISTS idx_items_category_id ON items(category_id);
CREATE INDEX IF NOT EXISTS idx_items_added_by ON items(added_by_user_id);
CREATE INDEX IF NOT EXISTS idx_items_purchased_by ON items(purchased_by_user_id);

-- Índices para Ordenação e Consulta de Histórico / Sugestões
CREATE INDEX IF NOT EXISTS idx_item_history_list_id ON item_history(list_id);
CREATE INDEX IF NOT EXISTS idx_item_history_suggestions ON item_history(list_id, purchase_count DESC, last_purchased_at DESC);

-- ==============================================================================
-- 9. Trigger para Atualização Automática de updated_at
-- ==============================================================================

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_users_updated_at ON users;
CREATE TRIGGER trg_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

DROP TRIGGER IF EXISTS trg_lists_updated_at ON lists;
CREATE TRIGGER trg_lists_updated_at
BEFORE UPDATE ON lists
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

DROP TRIGGER IF EXISTS trg_items_updated_at ON items;
CREATE TRIGGER trg_items_updated_at
BEFORE UPDATE ON items
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ==============================================================================
-- 10. Trigger para Sincronização do Histórico ao Marcar Item como Comprado
-- ==============================================================================

CREATE OR REPLACE FUNCTION handle_item_purchased_history()
RETURNS TRIGGER AS $$
DECLARE
    normalized_name TEXT;
BEGIN
    -- Dispara quando o item passa para 'purchased = TRUE'
    IF NEW.purchased = TRUE AND (OLD.purchased IS NULL OR OLD.purchased = FALSE) THEN
        normalized_name := LOWER(TRIM(NEW.name));
        
        INSERT INTO item_history (list_id, item_name_normalized, category_id, last_price, last_unit, purchase_count, last_purchased_at)
        VALUES (
            NEW.list_id,
            normalized_name,
            NEW.category_id,
            COALESCE(NEW.actual_price, NEW.estimated_price),
            NEW.unit,
            1,
            NOW()
        )
        ON CONFLICT (list_id, item_name_normalized)
        DO UPDATE SET
            purchase_count = item_history.purchase_count + 1,
            last_price = COALESCE(EXCLUDED.last_price, item_history.last_price),
            last_unit = EXCLUDED.last_unit,
            last_purchased_at = NOW();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_items_history_sync ON items;
CREATE TRIGGER trg_items_history_sync
AFTER UPDATE OF purchased ON items
FOR EACH ROW EXECUTE FUNCTION handle_item_purchased_history();

-- ==============================================================================
-- 11. Habilitação de Row-Level Security (RLS) para Segurança no Supabase
-- ==============================================================================

ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE lists ENABLE ROW LEVEL SECURITY;
ALTER TABLE list_members ENABLE ROW LEVEL SECURITY;
ALTER TABLE items ENABLE ROW LEVEL SECURITY;
ALTER TABLE item_history ENABLE ROW LEVEL SECURITY;
ALTER TABLE categories ENABLE ROW LEVEL SECURITY;

-- Categorias são legíveis por todos os usuários autenticados
DROP POLICY IF EXISTS "Categorias acessíveis publicamente" ON categories;
CREATE POLICY "Categorias acessíveis publicamente" ON categories
    FOR SELECT TO public USING (true);

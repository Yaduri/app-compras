-- ==============================================================================
-- Migração: 002_token_revocation.sql
-- Permite invalidar sessões (JWTs) emitidas antes de um instante.
-- Tokens com `iat` anterior a `tokens_valid_after` são recusados pela API.
-- ==============================================================================

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS tokens_valid_after TIMESTAMPTZ NOT NULL DEFAULT 'epoch';

-- Migration: 20260128194228_create_user_relationships
-- Description: User relationships graph (friends, blocks, requests)
-- Service: User Service
-- Author: Bulut

BEGIN;

-- =====================================================
-- ENUM TYPE
-- =====================================================
CREATE TYPE relationship_type AS ENUM (
    'friend',
    'blocked',
    'pending_incoming',
    'pending_outgoing'
);

-- =====================================================
-- TABLE
-- =====================================================
CREATE TABLE user_relationships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    type relationship_type NOT NULL,

    -- ✅ Message for friend requests (only for pending types)
    message TEXT,
    CONSTRAINT check_message_length CHECK (
        message IS NULL OR LENGTH(message) <= 200
    ),
    CONSTRAINT check_message_only_on_pending CHECK (
        (type IN ('pending_incoming', 'pending_outgoing') AND message IS NOT NULL)
        OR (type IN ('friend', 'blocked') AND message IS NULL)
        OR message IS NULL
    ),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- A user cannot create a relationship with themselves
    CONSTRAINT check_no_self_relationship CHECK (user_id <> target_user_id),

    -- Only one directed edge between two users
    CONSTRAINT unique_user_target_pair UNIQUE (user_id, target_user_id)
);

-- =====================================================
-- INDEXES (READ-HEAVY GRAPH OPTIMIZED)
-- =====================================================

-- All relationships of a user (main graph traversal)
CREATE INDEX idx_relationships_user
    ON user_relationships(user_id);

-- Reverse lookup (used for block checks, etc.)
CREATE INDEX idx_relationships_target
    ON user_relationships(target_user_id);

-- Optimized for querying a user's relationships by type
CREATE INDEX idx_relationships_user_type
    ON user_relationships(user_id, type);

-- Fast access to friend relationships only (partial index)
CREATE INDEX idx_relationships_friends
    ON user_relationships(user_id)
    WHERE type = 'friend';

-- Extremely fast block existence check
CREATE INDEX idx_relationships_blocks
    ON user_relationships(user_id, target_user_id)
    WHERE type = 'blocked';

-- Fast pending request lookups
CREATE INDEX idx_relationships_pending_incoming
    ON user_relationships(user_id)
    WHERE type = 'pending_incoming';

CREATE INDEX idx_relationships_pending_outgoing
    ON user_relationships(user_id)
    WHERE type = 'pending_outgoing';

-- =====================================================
-- ✅ BIDIRECTIONAL SYNC TRIGGER
-- Ensures consistency between user perspectives
-- =====================================================
CREATE OR REPLACE FUNCTION sync_bidirectional_relationship()
RETURNS TRIGGER AS $$
DECLARE
    reverse_type relationship_type;
    reverse_message TEXT;
BEGIN
    -- Determine reverse type and message handling
    CASE NEW.type
        WHEN 'pending_outgoing' THEN
            reverse_type := 'pending_incoming'::relationship_type;
            reverse_message := NEW.message;  -- Keep message on incoming side
        WHEN 'pending_incoming' THEN
            reverse_type := 'pending_outgoing'::relationship_type;
            reverse_message := NEW.message;
        WHEN 'friend' THEN
            reverse_type := 'friend'::relationship_type;
            reverse_message := NULL;
        WHEN 'blocked' THEN
            -- Blocks are unidirectional, no reverse relationship
            RETURN NEW;
    END CASE;

    -- Handle INSERT
    IF TG_OP = 'INSERT' THEN
        INSERT INTO user_relationships (user_id, target_user_id, type, message)
        VALUES (NEW.target_user_id, NEW.user_id, reverse_type, reverse_message)
        ON CONFLICT (user_id, target_user_id) DO UPDATE
            SET
                type = reverse_type,
                message = reverse_message,
                updated_at = NOW();

    -- Handle UPDATE
    ELSIF TG_OP = 'UPDATE' THEN
        UPDATE user_relationships
        SET
            type = reverse_type,
            message = reverse_message,
            updated_at = NOW()
        WHERE user_id = NEW.target_user_id
          AND target_user_id = NEW.user_id;

    -- Handle DELETE
    ELSIF TG_OP = 'DELETE' THEN
        -- Only delete reverse if it's not a block
        DELETE FROM user_relationships
        WHERE user_id = OLD.target_user_id
          AND target_user_id = OLD.user_id
          AND type != 'blocked';
        RETURN OLD;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_sync_bidirectional_relationship
    AFTER INSERT OR UPDATE OR DELETE ON user_relationships
    FOR EACH ROW
    EXECUTE FUNCTION sync_bidirectional_relationship();

-- =====================================================
-- ✅ PREVENT CONFLICTING RELATIONSHIPS
-- Can't have both friend and pending at the same time
-- =====================================================
CREATE OR REPLACE FUNCTION prevent_conflicting_relationships()
RETURNS TRIGGER AS $$
BEGIN
    -- Check for existing opposite direction relationship
    IF EXISTS (
        SELECT 1 FROM user_relationships
        WHERE user_id = NEW.target_user_id
          AND target_user_id = NEW.user_id
          AND (
              -- Can't send request if already friends
              (NEW.type IN ('pending_outgoing', 'pending_incoming') AND type = 'friend')
              OR
              -- Can't befriend if pending exists (should accept instead)
              (NEW.type = 'friend' AND type IN ('pending_incoming', 'pending_outgoing'))
          )
    ) THEN
        RAISE EXCEPTION 'Conflicting relationship exists';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_prevent_conflicts
    BEFORE INSERT OR UPDATE ON user_relationships
    FOR EACH ROW
    EXECUTE FUNCTION prevent_conflicting_relationships();

-- =====================================================
-- UPDATED_AT TRIGGER
-- =====================================================
CREATE OR REPLACE FUNCTION update_user_relationships_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_user_relationships_updated_at
    BEFORE UPDATE ON user_relationships
    FOR EACH ROW
    EXECUTE FUNCTION update_user_relationships_updated_at();

-- =====================================================
-- COMMENTS
-- =====================================================
COMMENT ON TABLE user_relationships IS 'Directed graph edges between users (friends, blocks, requests). Bidirectional sync enforced by triggers.';
COMMENT ON COLUMN user_relationships.user_id IS 'Owner of the relationship edge (perspective)';
COMMENT ON COLUMN user_relationships.target_user_id IS 'Target user of the relationship';
COMMENT ON COLUMN user_relationships.type IS 'Current relationship state from user_id perspective';
COMMENT ON COLUMN user_relationships.message IS 'Optional message for friend requests (only for pending types)';

COMMIT;
-- Migration: 20260128194228_create_friend_requests
-- Description: Create friendships table for user friends.
-- Service: User Service (primary)
-- Author: Bulut
-- Date: 2026-01-28

-- ============================================
-- 1. Enum Type for Status
-- ============================================
CREATE TYPE friend_request_status AS ENUM ('pending', 'accepted', 'declined');

-- ============================================
-- 2. Friend Requests Table
-- ============================================
CREATE TABLE friend_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Who sent the friend request
    sender_id UUID NOT NULL
        REFERENCES users(id) ON DELETE CASCADE,
    
    -- Who received the friend request
    receiver_id UUID NOT NULL
        REFERENCES users(id) ON DELETE CASCADE,
    
    -- Request status
    status friend_request_status NOT NULL DEFAULT 'pending',
    
    -- Optional: Custom message with request (like Discord)
    message TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT friend_requests_unique_pair UNIQUE (sender_id, receiver_id),
    CONSTRAINT friend_requests_no_self CHECK (sender_id <> receiver_id)
);

-- ============================================
-- 3. Indexes for Performance
-- ============================================

-- Find all requests sent by a user
CREATE INDEX idx_friend_requests_sender_id ON friend_requests(sender_id);

-- Find all requests received by a user
CREATE INDEX idx_friend_requests_receiver_id ON friend_requests(receiver_id);

-- Find pending requests for a user (most common query)
CREATE INDEX idx_friend_requests_receiver_status ON friend_requests(receiver_id, status);

-- Find sent pending requests
CREATE INDEX idx_friend_requests_sender_status ON friend_requests(sender_id, status);

-- Composite index for bidirectional check
CREATE INDEX idx_friend_requests_sender_receiver ON friend_requests(sender_id, receiver_id);

-- ============================================
-- 4. Updated_at Trigger
-- ============================================
CREATE OR REPLACE FUNCTION update_friend_requests_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_friend_requests_updated_at
    BEFORE UPDATE ON friend_requests
    FOR EACH ROW
    EXECUTE FUNCTION update_friend_requests_updated_at();

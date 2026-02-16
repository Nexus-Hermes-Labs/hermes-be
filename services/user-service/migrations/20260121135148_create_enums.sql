CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =====================================================
-- ENUM TYPES
-- =====================================================

CREATE TYPE user_status AS ENUM ('online', 'offline', 'idle', 'dnd');

CREATE TYPE dm_privacy AS ENUM (
    'everyone',
    'friends',
    'server_members',
    'none'
);

CREATE TYPE friend_request_privacy AS ENUM (
    'everyone',
    'friends_of_friends',
    'none'
);

CREATE TYPE relationship_type AS ENUM (
    'friend',
    'blocked',
    'pending_incoming',
    'pending_outgoing'
);

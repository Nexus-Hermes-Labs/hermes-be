-- Messaging service enums
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE message_type      AS ENUM ('text', 'system');
CREATE TYPE conversation_type AS ENUM ('dm', 'group_dm');

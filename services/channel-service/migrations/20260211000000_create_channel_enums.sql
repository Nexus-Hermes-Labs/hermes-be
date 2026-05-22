-- Create channel type enum
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE channel_type AS ENUM ('text', 'voice', 'category', 'announcement');

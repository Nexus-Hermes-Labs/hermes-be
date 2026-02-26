-- Messaging service enums
CREATE TYPE message_type      AS ENUM ('text', 'system');
CREATE TYPE conversation_type AS ENUM ('dm', 'group_dm');


-- Seed Data: Development Users
-- Description: Sample users for testing MVP
-- Author: Bulut
-- Date: 2026-01-21
-- Note: Only run in development/staging environments!

BEGIN;

-- Insert test users
-- Password for all users: "Password123!"
-- Hash generated with: argon2id (you'll need to replace with actual hashes)

INSERT INTO users (
    username,
    email,
    password_hash,
    display_name,
    avatar_url,
    bio,
    status,
    role,
    email_verified,
    is_active
) VALUES 
    -- Admin user
    (
        'admin',
        'admin@hermes.dev',
        '$argon2id$v=19$m=19456,t=2,p=1$dwK6T+IggC9sZuYb6pAXpA$AoTBQtDfBii8Xfa8EfT50HZ7Z4T8VlazuLSxoXT7Vfc',
        'System Admin',
        'https://api.dicebear.com/7.x/avataaars/svg?seed=admin',
        'System administrator account',
        'online',
        'admin',
        true,
        true
    ),
    -- Moderator user
    (
        'moderator',
        'mod@hermes.dev',
        '$argon2id$v=19$m=19456,t=2,p=1$dwK6T+IggC9sZuYb6pAXpA$AoTBQtDfBii8Xfa8EfT50HZ7Z4T8VlazuLSxoXT7Vfc',
        'Moderator User',
        'https://api.dicebear.com/7.x/avataaars/svg?seed=mod',
        'Community moderator',
        'online',
        'moderator',
        true,
        true
    ),
    -- Regular users
    (
        'alice',
        'alice@example.com',
        '$argon2id$v=19$m=19456,t=2,p=1$dwK6T+IggC9sZuYb6pAXpA$AoTBQtDfBii8Xfa8EfT50HZ7Z4T8VlazuLSxoXT7Vfc',
        'Alice Johnson',
        'https://api.dicebear.com/7.x/avataaars/svg?seed=alice',
        'Software engineer who loves coding 💻',
        'online',
        'user',
        true,
        true
    ),
    (
        'bob',
        'bob@example.com',
        '$argon2id$v=19$m=19456,t=2,p=1$dwK6T+IggC9sZuYb6pAXpA$AoTBQtDfBii8Xfa8EfT50HZ7Z4T8VlazuLSxoXT7Vfc',
        'Bob Smith',
        'https://api.dicebear.com/7.x/avataaars/svg?seed=bob',
        'Designer and UI/UX enthusiast 🎨',
        'idle',
        'user',
        true,
        true
    ),
    (
        'charlie',
        'charlie@example.com',
        '$argon2id$v=19$m=19456,t=2,p=1$dwK6T+IggC9sZuYb6pAXpA$AoTBQtDfBii8Xfa8EfT50HZ7Z4T8VlazuLSxoXT7Vfc',
        'Charlie Brown',
        'https://api.dicebear.com/7.x/avataaars/svg?seed=charlie',
        'Product manager and coffee lover ☕',
        'dnd',
        'user',
        true,
        true
    ),
    -- Unverified user
    (
        'newuser',
        'newuser@example.com',
        '$argon2id$v=19$m=19456,t=2,p=1$dwK6T+IggC9sZuYb6pAXpA$AoTBQtDfBii8Xfa8EfT50HZ7Z4T8VlazuLSxoXT7Vfc',
        'New User',
        NULL,
        NULL,
        'offline',
        'user',
        false, -- email not verified
        true
    ),
    -- Inactive user
    (
        'inactive',
        'inactive@example.com',
        '$argon2id$v=19$m=19456,t=2,p=1$dwK6T+IggC9sZuYb6pAXpA$AoTBQtDfBii8Xfa8EfT50HZ7Z4T8VlazuLSxoXT7Vfc',
        'Inactive User',
        NULL,
        'This account is disabled',
        'offline',
        'user',
        true,
        false -- account disabled
    );

-- Display created users
SELECT 
    username,
    email,
    display_name,
    role,
    status,
    email_verified,
    is_active,
    created_at
FROM users
ORDER BY role DESC, username;

COMMIT;

-- Reminder to generate proper password hashes
DO $$
BEGIN
    RAISE NOTICE '==================================================';
    RAISE NOTICE 'IMPORTANT: Replace placeholder password hashes!';
    RAISE NOTICE 'Default password for all users: SecurePassword123!';
    RAISE NOTICE '==================================================';
END $$;

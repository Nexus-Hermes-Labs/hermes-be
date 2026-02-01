
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
        '$argon2id$v=19$m=65536,t=3,p=1$pq6/PGqbN2GUB5BS8b1hNw$64G0iDW54eq0MZlOUw88oG+hWOfaW2Yj4DCVUaB9Et8',
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
        '$argon2id$v=19$m=65536,t=3,p=1$pq6/PGqbN2GUB5BS8b1hNw$64G0iDW54eq0MZlOUw88oG+hWOfaW2Yj4DCVUaB9Et8',
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
        '$argon2id$v=19$m=65536,t=3,p=1$pq6/PGqbN2GUB5BS8b1hNw$64G0iDW54eq0MZlOUw88oG+hWOfaW2Yj4DCVUaB9Et8',
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
        '$argon2id$v=19$m=65536,t=3,p=1$pq6/PGqbN2GUB5BS8b1hNw$64G0iDW54eq0MZlOUw88oG+hWOfaW2Yj4DCVUaB9Et8',
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
        '$argon2id$v=19$m=65536,t=3,p=1$pq6/PGqbN2GUB5BS8b1hNw$64G0iDW54eq0MZlOUw88oG+hWOfaW2Yj4DCVUaB9Et8',
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
        '$argon2id$v=19$m=65536,t=3,p=1$pq6/PGqbN2GUB5BS8b1hNw$64G0iDW54eq0MZlOUw88oG+hWOfaW2Yj4DCVUaB9Et8',
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
        '$argon2id$v=19$m=65536,t=3,p=1$pq6/PGqbN2GUB5BS8b1hNw$64G0iDW54eq0MZlOUw88oG+hWOfaW2Yj4DCVUaB9Et8',
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

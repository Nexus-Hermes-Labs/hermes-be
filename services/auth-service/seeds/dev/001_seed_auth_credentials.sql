-- =====================================================
-- SEED: AUTH_CREDENTIALS
-- =====================================================

BEGIN;

-- Password for all users: "Password123!"
-- Replace hashes in production

INSERT INTO auth_credentials (
    id,
    user_id,
    email,
    password_hash,
    email_verified,
    account_status,
    deleted_at,
    created_at,
    updated_at
)
VALUES

-- =====================================================
-- Admin User (Active) - Alice
-- =====================================================
(
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
    '11111111-1111-1111-1111-111111111111',
    'admin@example.com',
    '$2b$12$KIX6KXbE6o9P8hR1z8G5eOeVJZZzKzbwQ6vurIWBLL8GMDIS9Zh8a',
    TRUE,
    'active',
    NULL,
    NOW(),
    NOW()
),

-- =====================================================
-- Normal User 1 (Active) - Bob
-- =====================================================
(
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
    '22222222-2222-2222-2222-222222222222',
    'user1@example.com',
    '$2b$12$KIX6KXbE6o9P8hR1z8G5eOeVJZZzKzbwQ6vurIWBLL8GMDIS9Zh8a',
    TRUE,
    'active',
    NULL,
    NOW(),
    NOW()
),

-- =====================================================
-- Normal User 2 (Not Verified) - Charlie
-- =====================================================
(
    'cccccccc-cccc-cccc-cccc-cccccccccccc',
    '33333333-3333-3333-3333-333333333333',
    'user2@example.com',
    '$2b$12$KIX6KXbE6o9P8hR1z8G5eOeVJZZzKzbwQ6vurIWBLL8GMDIS9Zh8a',
    FALSE,
    'active',
    NULL,
    NOW(),
    NOW()
),

-- =====================================================
-- Suspended User - Diana
-- =====================================================
(
    'dddddddd-dddd-dddd-dddd-dddddddddddd',
    '44444444-4444-4444-4444-444444444444',
    'suspended@example.com',
    '$2b$12$KIX6KXbE6o9P8hR1z8G5eOeVJZZzKzbwQ6vurIWBLL8GMDIS9Zh8a',
    TRUE,
    'suspended',
    NULL,
    NOW(),
    NOW()
),

-- =====================================================
-- Deleted User - Eve
-- =====================================================
(
    'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee',
    '55555555-5555-5555-5555-555555555555',
    'deleted@example.com',
    '$2b$12$KIX6KXbE6o9P8hR1z8G5eOeVJZZzKzbwQ6vurIWBLL8GMDIS9Zh8a',
    TRUE,
    'deleted',
    NOW(),  -- deleted_at (REQUIRED)
    NOW(),
    NOW()
);

-- Verify seed
SELECT
    id,
    email,
    email_verified,
    account_status,
    deleted_at
FROM auth_credentials
ORDER BY email;

COMMIT;

DO $$
BEGIN
    RAISE NOTICE '==================================================';
    RAISE NOTICE 'Dev seed completed successfully.';
    RAISE NOTICE 'Default password: Password123!';
    RAISE NOTICE '==================================================';
END $$;
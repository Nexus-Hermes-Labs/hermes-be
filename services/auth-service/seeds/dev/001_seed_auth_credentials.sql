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
    '$argon2id$v=19$m=19456,t=2,p=1$8MCq+OmSvuK7Gxh2dC8dRw$FdyBDw1aCMxNpDQh1/0TvHTKy2QQshOrP0lL+TF3ZtU',
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
    '$argon2id$v=19$m=19456,t=2,p=1$8MCq+OmSvuK7Gxh2dC8dRw$FdyBDw1aCMxNpDQh1/0TvHTKy2QQshOrP0lL+TF3ZtU',
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
    '$argon2id$v=19$m=19456,t=2,p=1$8MCq+OmSvuK7Gxh2dC8dRw$FdyBDw1aCMxNpDQh1/0TvHTKy2QQshOrP0lL+TF3ZtU',
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
    '$argon2id$v=19$m=19456,t=2,p=1$8MCq+OmSvuK7Gxh2dC8dRw$FdyBDw1aCMxNpDQh1/0TvHTKy2QQshOrP0lL+TF3ZtU',
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
    '$argon2id$v=19$m=19456,t=2,p=1$8MCq+OmSvuK7Gxh2dC8dRw$FdyBDw1aCMxNpDQh1/0TvHTKy2QQshOrP0lL+TF3ZtU',
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
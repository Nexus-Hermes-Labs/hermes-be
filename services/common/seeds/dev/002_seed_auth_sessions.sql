-- =====================================================
-- SEED: AUTH_SESSIONS
-- =====================================================

BEGIN;

INSERT INTO auth_sessions (
    id,
    user_id,
    refresh_token_hash,
    ip_address,
    user_agent,
    device_name,
    expires_at,
    last_used_at,
    is_revoked,
    revoked_at,
    created_at
)
VALUES

-- =====================================================
-- Active Session - User1
-- =====================================================
(
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
    '22222222-2222-2222-2222-222222222222',
    'hashed_refresh_token_1',
    '127.0.0.1',
    'Mozilla/5.0',
    'MacBook Pro',
    NOW() + INTERVAL '30 days',
    NOW(),
    FALSE,
    NULL,
    NOW()
),

-- =====================================================
-- Active Session - User2
-- =====================================================
(
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
    '33333333-3333-3333-3333-333333333333',
    'hashed_refresh_token_2',
    '192.168.1.10',
    'Chrome',
    'Windows PC',
    NOW() + INTERVAL '15 days',
    NOW(),
    FALSE,
    NULL,
    NOW()
),

-- =====================================================
-- Revoked Session
-- =====================================================
(
    'cccccccc-cccc-cccc-cccc-cccccccccccc',
    '22222222-2222-2222-2222-222222222222',
    'hashed_refresh_token_3',
    '10.0.0.5',
    'Safari',
    'iPhone',
    NOW() + INTERVAL '30 days',
    NOW() - INTERVAL '1 day',
    TRUE,
    NOW() - INTERVAL '1 day',
    NOW() - INTERVAL '5 days'
),

-- =====================================================
-- Expired Session
-- =====================================================
(
    'dddddddd-dddd-dddd-dddd-dddddddddddd',
    '33333333-3333-3333-3333-333333333333',
    'hashed_refresh_token_4',
    '172.16.0.2',
    'Firefox',
    'Linux Laptop',
    NOW() - INTERVAL '1 day', -- expired
    NOW() - INTERVAL '2 days',
    FALSE,
    NULL,
    NOW() - INTERVAL '30 days'
);

-- Verify
SELECT
    id,
    user_id,
    expires_at,
    is_revoked,
    revoked_at
FROM auth_sessions
ORDER BY created_at;

COMMIT;

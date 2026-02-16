-- =====================================================
-- SEED: AUTH_SESSIONS
-- =====================================================

BEGIN;

INSERT INTO auth_sessions (
    id,
    credential_id,
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
-- Active Session - User1 (Bob)
-- =====================================================
(
    '11111111-2222-3333-4444-555555555555',
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
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
-- Active Session - User2 (Charlie)
-- =====================================================
(
    '22222222-3333-4444-5555-666666666666',
    'cccccccc-cccc-cccc-cccc-cccccccccccc',
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
-- Revoked Session - User1 (Bob)
-- =====================================================
(
    '33333333-4444-5555-6666-777777777777',
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
    'hashed_refresh_token_3',
    '10.0.0.5',
    'Safari',
    'iPhone',
    NOW(),
    NOW() - INTERVAL '1 day',
    TRUE,
    NOW() - INTERVAL '1 day',
    NOW() - INTERVAL '5 days'
),

-- =====================================================
-- Expired Session - User2 (Charlie)
-- =====================================================
(
    '44444444-5555-6666-7777-888888888888',
    'cccccccc-cccc-cccc-cccc-cccccccccccc',
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
    credential_id,
    expires_at,
    is_revoked,
    revoked_at
FROM auth_sessions
ORDER BY created_at;

COMMIT;
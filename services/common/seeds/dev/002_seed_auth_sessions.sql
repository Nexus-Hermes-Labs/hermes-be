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
-- Active Session - User1 (Bob)
-- =====================================================
(
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
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
    'cccccccc-cccc-cccc-cccc-cccccccccccc',
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
    'dddddddd-dddd-dddd-dddd-dddddddddddd',
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
    'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee',
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
    user_id,
    expires_at,
    is_revoked,
    revoked_at
FROM auth_sessions
ORDER BY created_at;

COMMIT;
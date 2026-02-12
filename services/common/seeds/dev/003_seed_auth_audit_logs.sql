-- =====================================================
-- SEED: AUTH_AUDIT_LOG
-- =====================================================

BEGIN;

INSERT INTO auth_audit_log (
    id,
    user_id,
    event_type,
    event_description,
    ip_address,
    user_agent,
    metadata,
    created_at
)
VALUES

-- =====================================================
-- USER 1 ACTIVITY FLOW
-- =====================================================

-- Login
(
    uuid_generate_v4(),
    '22222222-2222-2222-2222-222222222222',
    'login',
    'User logged in successfully',
    '127.0.0.1',
    'Mozilla/5.0 (Macintosh)',
    NULL,
    NOW() - INTERVAL '3 days'
),

-- Password Change
(
    uuid_generate_v4(),
    '22222222-2222-2222-2222-222222222222',
    'password_change',
    'User changed password',
    NULL,
    NULL,
    jsonb_build_object(
        'reason', 'user_initiated'
    ),
    NOW() - INTERVAL '2 days'
),

-- Logout
(
    uuid_generate_v4(),
    '22222222-2222-2222-2222-222222222222',
    'logout',
    'User logged out',
    '127.0.0.1',
    'Mozilla/5.0 (Macintosh)',
    NULL,
    NOW() - INTERVAL '2 days' + INTERVAL '2 hours'
),

-- =====================================================
-- USER 2 ACTIVITY FLOW
-- =====================================================

-- Email Change
(
    uuid_generate_v4(),
    '33333333-3333-3333-3333-333333333333',
    'email_change',
    'Email changed',
    NULL,
    NULL,
    jsonb_build_object(
        'old_email', 'olduser2@example.com',
        'new_email', 'user2@example.com'
    ),
    NOW() - INTERVAL '4 days'
),

-- Failed Login Attempts
(
    uuid_generate_v4(),
    '33333333-3333-3333-3333-333333333333',
    'failed_login',
    'Invalid password attempt',
    '192.168.1.10',
    'Chrome',
    jsonb_build_object(
        'attempt_number', 1
    ),
    NOW() - INTERVAL '6 hours'
),

(
    uuid_generate_v4(),
    '33333333-3333-3333-3333-333333333333',
    'failed_login',
    'Invalid password attempt',
    '192.168.1.10',
    'Chrome',
    jsonb_build_object(
        'attempt_number', 2
    ),
    NOW() - INTERVAL '5 hours'
),

-- Account Locked
(
    uuid_generate_v4(),
    '44444444-4444-4444-4444-444444444444',
    'account_locked',
    'Account locked due to failed login attempts',
    '192.168.1.15',
    'Firefox',
    jsonb_build_object(
        'failed_attempts', 5
    ),
    NOW() - INTERVAL '4 hours'
),

-- =====================================================
-- ADMIN ACTIVITY
-- =====================================================

(
    uuid_generate_v4(),
    '11111111-1111-1111-1111-111111111111',
    'admin_login',
    'Administrator logged in',
    '10.0.0.1',
    'PostmanRuntime/7.32.3',
    jsonb_build_object(
        'role', 'admin'
    ),
    NOW() - INTERVAL '1 day'
),

-- Deleted User Action
(
    uuid_generate_v4(),
    '55555555-5555-5555-5555-555555555555',
    'account_deleted',
    'User account soft deleted',
    NULL,
    NULL,
    jsonb_build_object(
        'deleted_by', 'system'
    ),
    NOW() - INTERVAL '7 days'
);

-- Verify chronological order
SELECT
    user_id,
    event_type,
    created_at
FROM auth_audit_log
ORDER BY created_at DESC;

COMMIT;

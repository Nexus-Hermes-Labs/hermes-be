-- =====================================================
-- SEED: AUTH_AUDIT_LOG
-- =====================================================

BEGIN;

INSERT INTO auth_audit_log (
    id,
    credential_id,
    event_type,
    event_description,
    ip_address,
    user_agent,
    metadata,
    created_at
)
VALUES

-- =====================================================
-- USER 1 (Bob) ACTIVITY FLOW
-- =====================================================

-- Login
(
    '10000000-0000-0000-0000-000000000001',
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
    'login',
    'User logged in successfully',
    '127.0.0.1',
    'Mozilla/5.0 (Macintosh)',
    NULL,
    NOW() - INTERVAL '3 days'
),

-- Password Change
(
    '20000000-0000-0000-0000-000000000002',
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
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
    '30000000-0000-0000-0000-000000000003',
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
    'logout',
    'User logged out',
    '127.0.0.1',
    'Mozilla/5.0 (Macintosh)',
    NULL,
    NOW() - INTERVAL '2 days' + INTERVAL '2 hours'
),

-- =====================================================
-- USER 2 (Charlie) ACTIVITY FLOW
-- =====================================================

-- Email Change
(
    '40000000-0000-0000-0000-000000000004',
    'cccccccc-cccc-cccc-cccc-cccccccccccc',
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
    '50000000-0000-0000-0000-000000000005',
    'cccccccc-cccc-cccc-cccc-cccccccccccc',
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
    '60000000-0000-0000-0000-000000000006',
    'cccccccc-cccc-cccc-cccc-cccccccccccc',
    'failed_login',
    'Invalid password attempt',
    '192.168.1.10',
    'Chrome',
    jsonb_build_object(
        'attempt_number', 2
    ),
    NOW() - INTERVAL '5 hours'
),

-- Account Locked (Diana - Suspended User)
(
    '70000000-0000-0000-0000-000000000007',
    'dddddddd-dddd-dddd-dddd-dddddddddddd',
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
-- ADMIN (Alice) ACTIVITY
-- =====================================================

(
    '80000000-0000-0000-0000-000000000008',
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
    'admin_login',
    'Administrator logged in',
    '10.0.0.1',
    'PostmanRuntime/7.32.3',
    jsonb_build_object(
        'role', 'admin'
    ),
    NOW() - INTERVAL '1 day'
),

-- Deleted User Action (Eve)
(
    '90000000-0000-0000-0000-000000000009',
    'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee',
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
    credential_id,
    event_type,
    created_at
FROM auth_audit_log
ORDER BY created_at DESC;

COMMIT;
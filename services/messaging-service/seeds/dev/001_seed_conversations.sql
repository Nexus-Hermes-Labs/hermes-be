-- =====================================================
-- SEED: CONVERSATIONS (DMs)
-- =====================================================
-- Users referenced (from user-service seeds):
--   Alice  : 11111111-1111-1111-1111-111111111111
--   Bob    : 22222222-2222-2222-2222-222222222222
--   Charlie: 33333333-3333-3333-3333-333333333333
--   Diana  : 44444444-4444-4444-4444-444444444444
--   Eve    : 55555555-5555-5555-5555-555555555555
-- =====================================================

BEGIN;

INSERT INTO conversations (id, type, created_at)
VALUES

-- DM: Alice ↔ Bob  (they are friends)
(
    'c0000001-0000-0000-0000-000000000001',
    'dm',
    NOW() - INTERVAL '25 days'
),

-- DM: Alice ↔ Charlie
(
    'c0000001-0000-0000-0000-000000000002',
    'dm',
    NOW() - INTERVAL '10 days'
),

-- Group DM: Alice + Bob + Charlie
(
    'c0000001-0000-0000-0000-000000000003',
    'group_dm',
    NOW() - INTERVAL '5 days'
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO conversation_members (conversation_id, user_id, joined_at)
VALUES

-- DM: Alice ↔ Bob
(
    'c0000001-0000-0000-0000-000000000001',
    '11111111-1111-1111-1111-111111111111',
    NOW() - INTERVAL '25 days'
),
(
    'c0000001-0000-0000-0000-000000000001',
    '22222222-2222-2222-2222-222222222222',
    NOW() - INTERVAL '25 days'
),

-- DM: Alice ↔ Charlie
(
    'c0000001-0000-0000-0000-000000000002',
    '11111111-1111-1111-1111-111111111111',
    NOW() - INTERVAL '10 days'
),
(
    'c0000001-0000-0000-0000-000000000002',
    '33333333-3333-3333-3333-333333333333',
    NOW() - INTERVAL '10 days'
),

-- Group DM: Alice + Bob + Charlie
(
    'c0000001-0000-0000-0000-000000000003',
    '11111111-1111-1111-1111-111111111111',
    NOW() - INTERVAL '5 days'
),
(
    'c0000001-0000-0000-0000-000000000003',
    '22222222-2222-2222-2222-222222222222',
    NOW() - INTERVAL '5 days'
),
(
    'c0000001-0000-0000-0000-000000000003',
    '33333333-3333-3333-3333-333333333333',
    NOW() - INTERVAL '5 days'
)
ON CONFLICT (conversation_id, user_id) DO NOTHING;

-- Verify seed
SELECT c.id, c.type, COUNT(cm.user_id) AS member_count
FROM conversations c
JOIN conversation_members cm ON cm.conversation_id = c.id
GROUP BY c.id, c.type
ORDER BY c.created_at;

COMMIT;

DO $$
BEGIN
    RAISE NOTICE '==================================================';
    RAISE NOTICE 'Messaging seed (conversations) completed successfully.';
    RAISE NOTICE '==================================================';
END $$;

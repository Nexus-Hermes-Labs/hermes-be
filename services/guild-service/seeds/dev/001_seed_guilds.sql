-- =====================================================
-- SEED: GUILDS
-- =====================================================
-- Users referenced (from user-service seeds):
--   Alice  : 11111111-1111-1111-1111-111111111111 (admin, active)
--   Bob    : 22222222-2222-2222-2222-222222222222 (active)
--   Charlie: 33333333-3333-3333-3333-333333333333 (active, unverified)
-- =====================================================

BEGIN;

INSERT INTO guilds (
    id,
    owner_id,
    name,
    description,
    icon_url,
    banner_url,
    visibility,
    member_count,
    max_members,
    created_at,
    updated_at,
    deleted_at
)
VALUES

-- =====================================================
-- Guild 1: Hermes HQ (public) - owned by Alice
-- =====================================================
(
    'aaaaaaaa-0001-0001-0001-000000000001',
    '11111111-1111-1111-1111-111111111111',
    'Hermes HQ',
    'The official Hermes platform community. Welcome to the hub!',
    'https://i.pravatar.cc/150?img=10',
    'https://i.pravatar.cc/400?img=10',
    'public',
    3,
    1000,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days',
    NULL
),

-- =====================================================
-- Guild 2: Dev Lounge (public) - owned by Bob
-- =====================================================
(
    'bbbbbbbb-0002-0002-0002-000000000002',
    '22222222-2222-2222-2222-222222222222',
    'Dev Lounge',
    'A place for developers to chill and talk shop.',
    'https://i.pravatar.cc/150?img=20',
    NULL,
    'public',
    2,
    500,
    NOW() - INTERVAL '14 days',
    NOW() - INTERVAL '14 days',
    NULL
),

-- =====================================================
-- Guild 3: Secret Society (private) - owned by Alice
-- =====================================================
(
    'cccccccc-0003-0003-0003-000000000003',
    '11111111-1111-1111-1111-111111111111',
    'Secret Society',
    'Invite-only club. Not for everyone.',
    'https://i.pravatar.cc/150?img=30',
    NULL,
    'private',
    1,
    50,
    NOW() - INTERVAL '7 days',
    NOW() - INTERVAL '7 days',
    NULL
);

-- Verify seed
SELECT id, name, visibility, owner_id, member_count FROM guilds ORDER BY created_at;

COMMIT;

DO $$
BEGIN
    RAISE NOTICE '==================================================';
    RAISE NOTICE 'Guild seed (guilds) completed successfully.';
    RAISE NOTICE '==================================================';
END $$;

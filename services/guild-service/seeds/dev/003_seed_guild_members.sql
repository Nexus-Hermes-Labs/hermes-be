-- =====================================================
-- SEED: GUILD_MEMBERS
-- =====================================================
-- Hermes HQ  : Alice (owner), Bob, Charlie
-- Dev Lounge : Bob (owner), Alice
-- Secret Soc : Alice (owner only)
-- =====================================================

BEGIN;

INSERT INTO guild_members (
    guild_id,
    user_id,
    nickname,
    joined_at,
    updated_at,
    left_at
)
VALUES

-- =====================================================
-- Hermes HQ members
-- =====================================================

-- Alice (owner)
(
    'aaaaaaaa-0001-0001-0001-000000000001',
    '11111111-1111-1111-1111-111111111111',
    NULL,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days',
    NULL
),

-- Bob
(
    'aaaaaaaa-0001-0001-0001-000000000001',
    '22222222-2222-2222-2222-222222222222',
    'Bobby',
    NOW() - INTERVAL '28 days',
    NOW() - INTERVAL '28 days',
    NULL
),

-- Charlie
(
    'aaaaaaaa-0001-0001-0001-000000000001',
    '33333333-3333-3333-3333-333333333333',
    NULL,
    NOW() - INTERVAL '20 days',
    NOW() - INTERVAL '20 days',
    NULL
),

-- =====================================================
-- Dev Lounge members
-- =====================================================

-- Bob (owner)
(
    'bbbbbbbb-0002-0002-0002-000000000002',
    '22222222-2222-2222-2222-222222222222',
    NULL,
    NOW() - INTERVAL '14 days',
    NOW() - INTERVAL '14 days',
    NULL
),

-- Alice
(
    'bbbbbbbb-0002-0002-0002-000000000002',
    '11111111-1111-1111-1111-111111111111',
    'AliceInDevland',
    NOW() - INTERVAL '10 days',
    NOW() - INTERVAL '10 days',
    NULL
),

-- =====================================================
-- Secret Society members
-- =====================================================

-- Alice (owner)
(
    'cccccccc-0003-0003-0003-000000000003',
    '11111111-1111-1111-1111-111111111111',
    NULL,
    NOW() - INTERVAL '7 days',
    NOW() - INTERVAL '7 days',
    NULL
);

-- Verify seed
SELECT guild_id, user_id, nickname, joined_at FROM guild_members ORDER BY guild_id, joined_at;

COMMIT;

DO $$
BEGIN
    RAISE NOTICE '==================================================';
    RAISE NOTICE 'Guild seed (guild_members) completed successfully.';
    RAISE NOTICE '==================================================';
END $$;

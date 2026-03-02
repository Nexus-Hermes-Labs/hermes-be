-- =====================================================
-- SEED: CHANNELS
-- =====================================================
-- Guilds referenced (from guild-service seeds):
--   Hermes HQ     : aaaaaaaa-0001-0001-0001-000000000001
--   Dev Lounge    : bbbbbbbb-0002-0002-0002-000000000002
--   Secret Society: cccccccc-0003-0003-0003-000000000003
-- =====================================================

BEGIN;

INSERT INTO channels (
    id,
    guild_id,
    parent_id,
    name,
    type,
    description,
    position,
    created_at,
    updated_at
)
VALUES

-- =====================================================
-- Hermes HQ channels
-- =====================================================

-- Category: Text Channels
(
    'aa100000-0001-0001-0001-000000000001',
    'aaaaaaaa-0001-0001-0001-000000000001',
    NULL,
    'Text Channels',
    'category',
    NULL,
    0,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days'
),

-- #general (child of Text Channels category)
(
    'aa100000-0001-0001-0001-000000000002',
    'aaaaaaaa-0001-0001-0001-000000000001',
    'aa100000-0001-0001-0001-000000000001',
    'general',
    'text',
    'General discussion for everything Hermes.',
    0,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days'
),

-- #announcements
(
    'aa100000-0001-0001-0001-000000000003',
    'aaaaaaaa-0001-0001-0001-000000000001',
    'aa100000-0001-0001-0001-000000000001',
    'announcements',
    'announcement',
    'Official announcements from the team.',
    1,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days'
),

-- #random
(
    'aa100000-0001-0001-0001-000000000004',
    'aaaaaaaa-0001-0001-0001-000000000001',
    'aa100000-0001-0001-0001-000000000001',
    'random',
    'text',
    'Off-topic chat and memes.',
    2,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days'
),

-- Category: Voice
(
    'aa100000-0001-0001-0001-000000000005',
    'aaaaaaaa-0001-0001-0001-000000000001',
    NULL,
    'Voice Channels',
    'category',
    NULL,
    1,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days'
),

-- General Voice
(
    'aa100000-0001-0001-0001-000000000006',
    'aaaaaaaa-0001-0001-0001-000000000001',
    'aa100000-0001-0001-0001-000000000005',
    'General',
    'voice',
    NULL,
    0,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days'
),

-- =====================================================
-- Dev Lounge channels
-- =====================================================

-- Category: Development
(
    'bb200000-0002-0002-0002-000000000001',
    'bbbbbbbb-0002-0002-0002-000000000002',
    NULL,
    'Development',
    'category',
    NULL,
    0,
    NOW() - INTERVAL '14 days',
    NOW() - INTERVAL '14 days'
),

-- #general
(
    'bb200000-0002-0002-0002-000000000002',
    'bbbbbbbb-0002-0002-0002-000000000002',
    'bb200000-0002-0002-0002-000000000001',
    'general',
    'text',
    'General developer chat.',
    0,
    NOW() - INTERVAL '14 days',
    NOW() - INTERVAL '14 days'
),

-- #code-review
(
    'bb200000-0002-0002-0002-000000000003',
    'bbbbbbbb-0002-0002-0002-000000000002',
    'bb200000-0002-0002-0002-000000000001',
    'code-review',
    'text',
    'Share your PRs and get feedback.',
    1,
    NOW() - INTERVAL '14 days',
    NOW() - INTERVAL '14 days'
),

-- #off-topic
(
    'bb200000-0002-0002-0002-000000000004',
    'bbbbbbbb-0002-0002-0002-000000000002',
    'bb200000-0002-0002-0002-000000000001',
    'off-topic',
    'text',
    'Anything that is not code.',
    2,
    NOW() - INTERVAL '14 days',
    NOW() - INTERVAL '14 days'
),

-- =====================================================
-- Secret Society channels
-- =====================================================

-- #secret-lair
(
    'cc300000-0003-0003-0003-000000000001',
    'cccccccc-0003-0003-0003-000000000003',
    NULL,
    'secret-lair',
    'text',
    'The inner sanctum. You are not supposed to be here.',
    0,
    NOW() - INTERVAL '7 days',
    NOW() - INTERVAL '7 days'
)
ON CONFLICT (id) DO NOTHING;

-- Verify seed
SELECT id, guild_id, name, type, position FROM channels ORDER BY guild_id, position, name;

COMMIT;

DO $$
BEGIN
    RAISE NOTICE '==================================================';
    RAISE NOTICE 'Channel seed completed successfully.';
    RAISE NOTICE '==================================================';
END $$;

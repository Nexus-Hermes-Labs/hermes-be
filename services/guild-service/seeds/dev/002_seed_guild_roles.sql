-- =====================================================
-- SEED: GUILD_ROLES
-- =====================================================
-- Each guild gets a default @everyone role plus
-- service-specific named roles.
-- Permission bits are illustrative (not enforced yet).
-- =====================================================

BEGIN;

INSERT INTO guild_roles (
    id,
    guild_id,
    name,
    color,
    permissions,
    position,
    hoist,
    mentionable,
    is_default,
    created_at,
    updated_at
)
VALUES

-- =====================================================
-- Hermes HQ (aaaaaaaa-0001-...) roles
-- =====================================================

-- @everyone (default, lowest position)
(
    '11111111-0001-0001-0001-000000000001',
    'aaaaaaaa-0001-0001-0001-000000000001',
    '@everyone',
    0,
    104324161,  -- read messages, send messages, add reactions
    0,
    FALSE,
    FALSE,
    TRUE,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days'
),

-- Member
(
    '11111111-0001-0001-0001-000000000002',
    'aaaaaaaa-0001-0001-0001-000000000001',
    'Member',
    3447003,    -- blue
    104324161,
    1,
    TRUE,
    TRUE,
    FALSE,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days'
),

-- Admin
(
    '11111111-0001-0001-0001-000000000003',
    'aaaaaaaa-0001-0001-0001-000000000001',
    'Admin',
    15158332,   -- red
    8,          -- administrator bit
    2,
    TRUE,
    TRUE,
    FALSE,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days'
),

-- =====================================================
-- Dev Lounge (bbbbbbbb-0002-...) roles
-- =====================================================

-- @everyone (default)
(
    '11111111-0002-0002-0002-000000000001',
    'bbbbbbbb-0002-0002-0002-000000000002',
    '@everyone',
    0,
    104324161,
    0,
    FALSE,
    FALSE,
    TRUE,
    NOW() - INTERVAL '14 days',
    NOW() - INTERVAL '14 days'
),

-- Developer
(
    '11111111-0002-0002-0002-000000000002',
    'bbbbbbbb-0002-0002-0002-000000000002',
    'Developer',
    5763719,    -- green
    104324161,
    1,
    TRUE,
    TRUE,
    FALSE,
    NOW() - INTERVAL '14 days',
    NOW() - INTERVAL '14 days'
),

-- =====================================================
-- Secret Society (cccccccc-0003-...) roles
-- =====================================================

-- @everyone (default)
(
    '11111111-0003-0003-0003-000000000001',
    'cccccccc-0003-0003-0003-000000000003',
    '@everyone',
    0,
    104324161,
    0,
    FALSE,
    FALSE,
    TRUE,
    NOW() - INTERVAL '7 days',
    NOW() - INTERVAL '7 days'
),

-- Insider
(
    '11111111-0003-0003-0003-000000000002',
    'cccccccc-0003-0003-0003-000000000003',
    'Insider',
    16776960,   -- yellow
    104324161,
    1,
    TRUE,
    FALSE,
    FALSE,
    NOW() - INTERVAL '7 days',
    NOW() - INTERVAL '7 days'
);

-- Verify seed
SELECT id, guild_id, name, position, is_default FROM guild_roles ORDER BY guild_id, position;

COMMIT;

DO $$
BEGIN
    RAISE NOTICE '==================================================';
    RAISE NOTICE 'Guild seed (guild_roles) completed successfully.';
    RAISE NOTICE '==================================================';
END $$;

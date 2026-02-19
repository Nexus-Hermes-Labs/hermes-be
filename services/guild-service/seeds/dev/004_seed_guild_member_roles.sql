-- =====================================================
-- SEED: GUILD_MEMBER_ROLES
-- =====================================================
-- Assigns roles to guild members.
-- Every member gets @everyone implicitly via is_default,
-- but we also track it explicitly here for the junction table.
-- =====================================================

BEGIN;

INSERT INTO guild_member_roles (
    guild_id,
    user_id,
    role_id,
    assigned_at
)
VALUES

-- =====================================================
-- Hermes HQ role assignments
-- =====================================================

-- Alice -> @everyone + Admin
(
    'aaaaaaaa-0001-0001-0001-000000000001',
    '11111111-1111-1111-1111-111111111111',
    '11111111-0001-0001-0001-000000000001',  -- @everyone
    NOW() - INTERVAL '30 days'
),
(
    'aaaaaaaa-0001-0001-0001-000000000001',
    '11111111-1111-1111-1111-111111111111',
    '11111111-0001-0001-0001-000000000003',  -- Admin
    NOW() - INTERVAL '30 days'
),

-- Bob -> @everyone + Member
(
    'aaaaaaaa-0001-0001-0001-000000000001',
    '22222222-2222-2222-2222-222222222222',
    '11111111-0001-0001-0001-000000000001',  -- @everyone
    NOW() - INTERVAL '28 days'
),
(
    'aaaaaaaa-0001-0001-0001-000000000001',
    '22222222-2222-2222-2222-222222222222',
    '11111111-0001-0001-0001-000000000002',  -- Member
    NOW() - INTERVAL '28 days'
),

-- Charlie -> @everyone + Member
(
    'aaaaaaaa-0001-0001-0001-000000000001',
    '33333333-3333-3333-3333-333333333333',
    '11111111-0001-0001-0001-000000000001',  -- @everyone
    NOW() - INTERVAL '20 days'
),
(
    'aaaaaaaa-0001-0001-0001-000000000001',
    '33333333-3333-3333-3333-333333333333',
    '11111111-0001-0001-0001-000000000002',  -- Member
    NOW() - INTERVAL '20 days'
),

-- =====================================================
-- Dev Lounge role assignments
-- =====================================================

-- Bob -> @everyone + Developer
(
    'bbbbbbbb-0002-0002-0002-000000000002',
    '22222222-2222-2222-2222-222222222222',
    '11111111-0002-0002-0002-000000000001',  -- @everyone
    NOW() - INTERVAL '14 days'
),
(
    'bbbbbbbb-0002-0002-0002-000000000002',
    '22222222-2222-2222-2222-222222222222',
    '11111111-0002-0002-0002-000000000002',  -- Developer
    NOW() - INTERVAL '14 days'
),

-- Alice -> @everyone + Developer
(
    'bbbbbbbb-0002-0002-0002-000000000002',
    '11111111-1111-1111-1111-111111111111',
    '11111111-0002-0002-0002-000000000001',  -- @everyone
    NOW() - INTERVAL '10 days'
),
(
    'bbbbbbbb-0002-0002-0002-000000000002',
    '11111111-1111-1111-1111-111111111111',
    '11111111-0002-0002-0002-000000000002',  -- Developer
    NOW() - INTERVAL '10 days'
),

-- =====================================================
-- Secret Society role assignments
-- =====================================================

-- Alice -> @everyone + Insider
(
    'cccccccc-0003-0003-0003-000000000003',
    '11111111-1111-1111-1111-111111111111',
    '11111111-0003-0003-0003-000000000001',  -- @everyone
    NOW() - INTERVAL '7 days'
),
(
    'cccccccc-0003-0003-0003-000000000003',
    '11111111-1111-1111-1111-111111111111',
    '11111111-0003-0003-0003-000000000002',  -- Insider
    NOW() - INTERVAL '7 days'
);

-- Verify seed
SELECT guild_id, user_id, role_id, assigned_at FROM guild_member_roles ORDER BY guild_id, user_id, assigned_at;

COMMIT;

DO $$
BEGIN
    RAISE NOTICE '==================================================';
    RAISE NOTICE 'Guild seed (guild_member_roles) completed successfully.';
    RAISE NOTICE '==================================================';
END $$;

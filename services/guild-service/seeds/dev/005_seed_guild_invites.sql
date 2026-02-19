-- =====================================================
-- SEED: GUILD_INVITES
-- =====================================================

BEGIN;

INSERT INTO guild_invites (
    code,
    guild_id,
    creator_id,
    max_uses,
    uses,
    expires_at,
    revoked,
    created_at
)
VALUES

-- Active invite to Hermes HQ (created by Alice, unlimited uses)
(
    'HERMES01',
    'aaaaaaaa-0001-0001-0001-000000000001',
    '11111111-1111-1111-1111-111111111111',
    NULL,
    5,
    NULL,
    FALSE,
    NOW() - INTERVAL '25 days'
),

-- Limited invite to Hermes HQ (created by Alice, 10-use cap, already 3 used)
(
    'HERMES02',
    'aaaaaaaa-0001-0001-0001-000000000001',
    '11111111-1111-1111-1111-111111111111',
    10,
    3,
    NOW() + INTERVAL '5 days',
    FALSE,
    NOW() - INTERVAL '10 days'
),

-- Active invite to Dev Lounge (created by Bob)
(
    'DEVLNG01',
    'bbbbbbbb-0002-0002-0002-000000000002',
    '22222222-2222-2222-2222-222222222222',
    50,
    1,
    NOW() + INTERVAL '30 days',
    FALSE,
    NOW() - INTERVAL '12 days'
),

-- Revoked invite to Dev Lounge (created by Bob)
(
    'DEVLNG02',
    'bbbbbbbb-0002-0002-0002-000000000002',
    '22222222-2222-2222-2222-222222222222',
    5,
    2,
    NOW() + INTERVAL '10 days',
    TRUE,
    NOW() - INTERVAL '13 days'
),

-- Expired invite to Secret Society (created by Alice)
(
    'SCRT0001',
    'cccccccc-0003-0003-0003-000000000003',
    '11111111-1111-1111-1111-111111111111',
    1,
    0,
    NOW() - INTERVAL '1 day',
    FALSE,
    NOW() - INTERVAL '8 days'
);

-- Verify seed
SELECT code, guild_id, creator_id, max_uses, uses, expires_at, revoked FROM guild_invites ORDER BY created_at;

COMMIT;

DO $$
BEGIN
    RAISE NOTICE '==================================================';
    RAISE NOTICE 'Guild seed (guild_invites) completed successfully.';
    RAISE NOTICE '==================================================';
END $$;

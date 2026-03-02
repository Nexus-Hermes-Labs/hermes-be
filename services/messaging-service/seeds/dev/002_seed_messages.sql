-- =====================================================
-- SEED: MESSAGES
-- =====================================================
-- Channels (from channel-service seeds):
--   Hermes HQ  #general      : aa100000-0001-0001-0001-000000000002
--   Hermes HQ  #announcements: aa100000-0001-0001-0001-000000000003
--   Hermes HQ  #random       : aa100000-0001-0001-0001-000000000004
--   Dev Lounge #general      : bb200000-0002-0002-0002-000000000002
--   Dev Lounge #code-review  : bb200000-0002-0002-0002-000000000003
--   Dev Lounge #off-topic    : bb200000-0002-0002-0002-000000000004
--   Secret Soc #secret-lair  : cc300000-0003-0003-0003-000000000001
-- Conversations (from seed 001):
--   Alice ↔ Bob DM    : c0000001-0000-0000-0000-000000000001
--   Alice ↔ Charlie DM: c0000001-0000-0000-0000-000000000002
--   Group DM (A+B+C)  : c0000001-0000-0000-0000-000000000003
-- Users:
--   Alice  : 11111111-1111-1111-1111-111111111111
--   Bob    : 22222222-2222-2222-2222-222222222222
--   Charlie: 33333333-3333-3333-3333-333333333333
-- Message ID scheme (valid hex UUIDs):
--   1xxxxxxx-0001-... = Hermes HQ #general
--   2xxxxxxx-0001-... = Hermes HQ #announcements
--   3xxxxxxx-0001-... = Hermes HQ #random
--   4xxxxxxx-0002-... = Dev Lounge #general
--   5xxxxxxx-0002-... = Dev Lounge #code-review
--   6xxxxxxx-0002-... = Dev Lounge #off-topic
--   7xxxxxxx-0003-... = Secret Society #secret-lair
--   8xxxxxxx-0001-... = DM Alice ↔ Bob
--   9xxxxxxx-0001-... = DM Alice ↔ Charlie
--   a0000000-0000-... = Group DM
-- =====================================================

BEGIN;

INSERT INTO messages (
    id,
    channel_id,
    conversation_id,
    user_id,
    content,
    type,
    reply_to_id,
    is_deleted,
    created_at,
    updated_at
)
VALUES

-- =====================================================
-- Hermes HQ #general (channel aa100000-0001-0001-0001-000000000002)
-- =====================================================
(
    '10000001-0001-0001-0001-000000000001',
    'aa100000-0001-0001-0001-000000000002',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Welcome everyone to Hermes HQ! 🎉 This is our main community server.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '30 days',
    NOW() - INTERVAL '30 days'
),
(
    '10000001-0001-0001-0001-000000000002',
    'aa100000-0001-0001-0001-000000000002',
    NULL,
    '22222222-2222-2222-2222-222222222222', -- Bob
    'Hey everyone! Excited to be here. This platform is looking great Alice.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '28 days',
    NOW() - INTERVAL '28 days'
),
(
    '10000001-0001-0001-0001-000000000003',
    'aa100000-0001-0001-0001-000000000002',
    NULL,
    '33333333-3333-3333-3333-333333333333', -- Charlie
    'Just joined, this is awesome!',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '20 days',
    NOW() - INTERVAL '20 days'
),
(
    '10000001-0001-0001-0001-000000000004',
    'aa100000-0001-0001-0001-000000000002',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Thanks Bob! We have been working hard on it. Feel free to explore all the channels.',
    'text',
    '10000001-0001-0001-0001-000000000002', -- reply to Bob
    FALSE,
    NOW() - INTERVAL '20 days',
    NOW() - INTERVAL '20 days'
),
(
    '10000001-0001-0001-0001-000000000005',
    'aa100000-0001-0001-0001-000000000002',
    NULL,
    '22222222-2222-2222-2222-222222222222', -- Bob
    'What is the tech stack for this? I see Rust on the backend.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '15 days',
    NOW() - INTERVAL '15 days'
),
(
    '10000001-0001-0001-0001-000000000006',
    'aa100000-0001-0001-0001-000000000002',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Rust microservices with Axum, PostgreSQL, Redis, NATS for events, and React on the frontend. All running on Docker with Traefik as the edge proxy.',
    'text',
    '10000001-0001-0001-0001-000000000005',
    FALSE,
    NOW() - INTERVAL '15 days',
    NOW() - INTERVAL '15 days'
),
(
    '10000001-0001-0001-0001-000000000007',
    'aa100000-0001-0001-0001-000000000002',
    NULL,
    '22222222-2222-2222-2222-222222222222', -- Bob
    'That sounds amazing. Real-time via WebSockets?',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '14 days',
    NOW() - INTERVAL '14 days'
),
(
    '10000001-0001-0001-0001-000000000008',
    'aa100000-0001-0001-0001-000000000002',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Exactly! We have a dedicated realtime-service. NATS fans out events to all connected WS clients.',
    'text',
    '10000001-0001-0001-0001-000000000007',
    FALSE,
    NOW() - INTERVAL '14 days',
    NOW() - INTERVAL '14 days'
),

-- =====================================================
-- Hermes HQ #announcements (channel aa100000-0001-0001-0001-000000000003)
-- =====================================================
(
    '20000001-0001-0001-0001-000000000001',
    'aa100000-0001-0001-0001-000000000003',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Hermes v0.1.0 is live! We now support guild channels, direct messages, and real-time updates.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '7 days',
    NOW() - INTERVAL '7 days'
),
(
    '20000001-0001-0001-0001-000000000002',
    'aa100000-0001-0001-0001-000000000003',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Upcoming: voice channels, message search, and AI-powered translation. Stay tuned!',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '3 days',
    NOW() - INTERVAL '3 days'
),

-- =====================================================
-- Hermes HQ #random (channel aa100000-0001-0001-0001-000000000004)
-- =====================================================
(
    '30000001-0001-0001-0001-000000000001',
    'aa100000-0001-0001-0001-000000000004',
    NULL,
    '22222222-2222-2222-2222-222222222222', -- Bob
    'What is everyone working on today?',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '12 days',
    NOW() - INTERVAL '12 days'
),
(
    '30000001-0001-0001-0001-000000000002',
    'aa100000-0001-0001-0001-000000000004',
    NULL,
    '33333333-3333-3333-3333-333333333333', -- Charlie
    'Reviewing some PRs and drinking way too much coffee',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '12 days',
    NOW() - INTERVAL '12 days'
),
(
    '30000001-0001-0001-0001-000000000003',
    'aa100000-0001-0001-0001-000000000004',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Debugging the channel-service pagination. Almost got it!',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '11 days',
    NOW() - INTERVAL '11 days'
),

-- =====================================================
-- Dev Lounge #general (channel bb200000-0002-0002-0002-000000000002)
-- =====================================================
(
    '40000002-0002-0002-0002-000000000001',
    'bb200000-0002-0002-0002-000000000002',
    NULL,
    '22222222-2222-2222-2222-222222222222', -- Bob
    'Welcome to Dev Lounge! This is the place for developers to hang out.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '14 days',
    NOW() - INTERVAL '14 days'
),
(
    '40000002-0002-0002-0002-000000000002',
    'bb200000-0002-0002-0002-000000000002',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Love the vibe here already. Anyone else doing Rust these days?',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '10 days',
    NOW() - INTERVAL '10 days'
),
(
    '40000002-0002-0002-0002-000000000003',
    'bb200000-0002-0002-0002-000000000002',
    NULL,
    '22222222-2222-2222-2222-222222222222', -- Bob
    'I have been learning it for the last 6 months. The borrow checker is tough at first but it clicks eventually.',
    'text',
    '40000002-0002-0002-0002-000000000002',
    FALSE,
    NOW() - INTERVAL '10 days',
    NOW() - INTERVAL '10 days'
),
(
    '40000002-0002-0002-0002-000000000004',
    'bb200000-0002-0002-0002-000000000002',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Exactly! Once it clicks you never want to go back. No more null pointer exceptions!',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '9 days',
    NOW() - INTERVAL '9 days'
),

-- =====================================================
-- Dev Lounge #code-review (channel bb200000-0002-0002-0002-000000000003)
-- =====================================================
(
    '50000002-0002-0002-0002-000000000001',
    'bb200000-0002-0002-0002-000000000003',
    NULL,
    '22222222-2222-2222-2222-222222222222', -- Bob
    'Share your PRs here and we will review them!',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '13 days',
    NOW() - INTERVAL '13 days'
),
(
    '50000002-0002-0002-0002-000000000002',
    'bb200000-0002-0002-0002-000000000003',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Just opened a PR for the realtime WebSocket gateway. Would love some eyes on the NATS fan-out logic.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '4 days',
    NOW() - INTERVAL '4 days'
),
(
    '50000002-0002-0002-0002-000000000003',
    'bb200000-0002-0002-0002-000000000003',
    NULL,
    '22222222-2222-2222-2222-222222222222', -- Bob
    'On it! The DashMap usage looks clean. One question: do we need to worry about stale entries in MsgContextCache?',
    'text',
    '50000002-0002-0002-0002-000000000002',
    FALSE,
    NOW() - INTERVAL '3 days',
    NOW() - INTERVAL '3 days'
),
(
    '50000002-0002-0002-0002-000000000004',
    'bb200000-0002-0002-0002-000000000003',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Good catch! I will add a TTL-based eviction in the next iteration. For now it is bounded by message lifetime.',
    'text',
    '50000002-0002-0002-0002-000000000003',
    FALSE,
    NOW() - INTERVAL '3 days',
    NOW() - INTERVAL '3 days'
),

-- =====================================================
-- Dev Lounge #off-topic (channel bb200000-0002-0002-0002-000000000004)
-- =====================================================
(
    '60000002-0002-0002-0002-000000000001',
    'bb200000-0002-0002-0002-000000000004',
    NULL,
    '22222222-2222-2222-2222-222222222222', -- Bob
    'Hot take: tabs are better than spaces.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '8 days',
    NOW() - INTERVAL '8 days'
),
(
    '60000002-0002-0002-0002-000000000002',
    'bb200000-0002-0002-0002-000000000004',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Bold stance. You are wrong, but I respect the confidence.',
    'text',
    '60000002-0002-0002-0002-000000000001',
    FALSE,
    NOW() - INTERVAL '8 days',
    NOW() - INTERVAL '8 days'
),

-- =====================================================
-- Secret Society #secret-lair (channel cc300000-0003-0003-0003-000000000001)
-- =====================================================
(
    '70000003-0003-0003-0003-000000000001',
    'cc300000-0003-0003-0003-000000000001',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'If you are reading this, you found the secret channel. Welcome.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '7 days',
    NOW() - INTERVAL '7 days'
),
(
    '70000003-0003-0003-0003-000000000002',
    'cc300000-0003-0003-0003-000000000001',
    NULL,
    '11111111-1111-1111-1111-111111111111', -- Alice
    'This is where we plan world domination. Or just test private channels.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '6 days',
    NOW() - INTERVAL '6 days'
),

-- =====================================================
-- DM: Alice ↔ Bob (c0000001-0000-0000-0000-000000000001)
-- =====================================================
(
    '80000001-0000-0000-0000-000000000001',
    NULL,
    'c0000001-0000-0000-0000-000000000001',
    '22222222-2222-2222-2222-222222222222', -- Bob
    'Hey Alice! Great work on the Hermes platform.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '25 days',
    NOW() - INTERVAL '25 days'
),
(
    '80000001-0000-0000-0000-000000000002',
    NULL,
    'c0000001-0000-0000-0000-000000000001',
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Thanks Bob! It has been a big effort. How are things on your end?',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '24 days',
    NOW() - INTERVAL '24 days'
),
(
    '80000001-0000-0000-0000-000000000003',
    NULL,
    'c0000001-0000-0000-0000-000000000001',
    '22222222-2222-2222-2222-222222222222', -- Bob
    'Pretty good! Been diving into async Rust more. Tokio is incredible.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '24 days',
    NOW() - INTERVAL '24 days'
),
(
    '80000001-0000-0000-0000-000000000004',
    NULL,
    'c0000001-0000-0000-0000-000000000001',
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Agreed. The whole ecosystem is maturing so fast. Want to catch up for a voice chat later?',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '2 days',
    NOW() - INTERVAL '2 days'
),
(
    '80000001-0000-0000-0000-000000000005',
    NULL,
    'c0000001-0000-0000-0000-000000000001',
    '22222222-2222-2222-2222-222222222222', -- Bob
    'Absolutely, sounds good!',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '2 days',
    NOW() - INTERVAL '2 days'
),

-- =====================================================
-- DM: Alice ↔ Charlie (c0000001-0000-0000-0000-000000000002)
-- =====================================================
(
    '90000002-0000-0000-0000-000000000001',
    NULL,
    'c0000001-0000-0000-0000-000000000002',
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Hey Charlie, glad you joined the server!',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '10 days',
    NOW() - INTERVAL '10 days'
),
(
    '90000002-0000-0000-0000-000000000002',
    NULL,
    'c0000001-0000-0000-0000-000000000002',
    '33333333-3333-3333-3333-333333333333', -- Charlie
    'Thanks Alice! It looks really polished already.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '10 days',
    NOW() - INTERVAL '10 days'
),

-- =====================================================
-- Group DM: Alice + Bob + Charlie (c0000001-0000-0000-0000-000000000003)
-- =====================================================
(
    'a0000003-0000-0000-0000-000000000001',
    NULL,
    'c0000001-0000-0000-0000-000000000003',
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Created this group so we can coordinate the next sprint!',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '5 days',
    NOW() - INTERVAL '5 days'
),
(
    'a0000003-0000-0000-0000-000000000002',
    NULL,
    'c0000001-0000-0000-0000-000000000003',
    '22222222-2222-2222-2222-222222222222', -- Bob
    'Nice! I am taking the voice channel work.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '5 days',
    NOW() - INTERVAL '5 days'
),
(
    'a0000003-0000-0000-0000-000000000003',
    NULL,
    'c0000001-0000-0000-0000-000000000003',
    '33333333-3333-3333-3333-333333333333', -- Charlie
    'I can help with search-service. Been looking at meilisearch integration.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '4 days',
    NOW() - INTERVAL '4 days'
),
(
    'a0000003-0000-0000-0000-000000000004',
    NULL,
    'c0000001-0000-0000-0000-000000000003',
    '11111111-1111-1111-1111-111111111111', -- Alice
    'Perfect. I will focus on AI translation. Let us sync again in two days.',
    'text',
    NULL,
    FALSE,
    NOW() - INTERVAL '4 days',
    NOW() - INTERVAL '4 days'
)
ON CONFLICT (id) DO NOTHING;

-- Verify seed
SELECT
    COALESCE(channel_id::TEXT, 'conv:'||conversation_id::TEXT) AS context,
    COUNT(*) AS message_count
FROM messages
GROUP BY context
ORDER BY context;

COMMIT;

DO $$
BEGIN
    RAISE NOTICE '==================================================';
    RAISE NOTICE 'Messaging seed (messages) completed successfully.';
    RAISE NOTICE '==================================================';
END $$;

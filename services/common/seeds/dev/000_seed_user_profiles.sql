BEGIN;

-- Insert into user_profiles with FIXED UUIDs
INSERT INTO user_profiles (id, username, display_name, avatar_url, bio, status, custom_status_text, custom_status_emoji, last_seen_at)
VALUES
    ('11111111-1111-1111-1111-111111111111', 'alice', 'Alice Smith', 'https://i.pravatar.cc/150?img=1', 'Enthusiastic explorer.', 'online', NULL, NULL, NOW()),
    ('22222222-2222-2222-2222-222222222222', 'bob', 'Bob Johnson', 'https://i.pravatar.cc/150?img=2', 'Loves coding and coffee.', 'idle', 'Thinking...', '🤔', NOW() - INTERVAL '1 hour'),
    ('33333333-3333-3333-3333-333333333333', 'charlie', 'Charlie Brown', 'https://i.pravatar.cc/150?img=3', 'Just chilling.', 'offline', NULL, NULL, NOW() - INTERVAL '2 days'),
    ('44444444-4444-4444-4444-444444444444', 'diana', 'Diana Prince', 'https://i.pravatar.cc/150?img=4', 'Wondering around.', 'dnd', 'Busy saving the world.', '🦸‍♀️', NOW()),
    ('55555555-5555-5555-5555-555555555555', 'eve', 'Eve Adams', 'https://i.pravatar.cc/150?img=5', 'Always learning new things.', 'online', NULL, NULL, NOW());

-- Insert into user_badges
INSERT INTO user_badges (user_id, badge_type, badge_name, badge_description, badge_icon_url, display_order)
VALUES
    ('11111111-1111-1111-1111-111111111111', 'developer', 'Early Contributor', 'Awarded to early project contributors.', 'https://example.com/badge_dev.png', 1),
    ('22222222-2222-2222-2222-222222222222', 'supporter', 'Gold Supporter', 'Thanks for your generous support!', 'https://example.com/badge_gold.png', 1),
    ('55555555-5555-5555-5555-555555555555', 'tester', 'Beta Tester', 'Helped us find bugs!', 'https://example.com/badge_beta.png', 1);

-- Insert into user_relationships
-- Alice (friend) Bob
INSERT INTO user_relationships (user_id, target_user_id, type)
VALUES ('11111111-1111-1111-1111-111111111111', '22222222-2222-2222-2222-222222222222', 'friend');

-- Charlie (pending outgoing) Diana
INSERT INTO user_relationships (user_id, target_user_id, type, message)
VALUES ('33333333-3333-3333-3333-333333333333', '44444444-4444-4444-4444-444444444444', 'pending_outgoing', 'Hey, wanna connect?');

-- Eve (blocked) Bob
INSERT INTO user_relationships (user_id, target_user_id, type)
VALUES ('55555555-5555-5555-5555-555555555555', '22222222-2222-2222-2222-222222222222', 'blocked');

COMMIT;
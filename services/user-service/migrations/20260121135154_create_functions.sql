-- =====================================================
-- COMMON FUNCTIONS
-- =====================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION create_default_privacy_settings()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO user_privacy_settings (user_id)
    VALUES (NEW.id) ON CONFLICT (user_id) DO NOTHING;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_last_seen_on_status_change()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status IS DISTINCT FROM NEW.status THEN
        NEW.last_seen_at = NOW();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION clean_expired_custom_status()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.custom_status_expires_at IS NOT NULL
       AND NEW.custom_status_expires_at <= NOW() THEN
        NEW.custom_status_text = NULL;
        NEW.custom_status_emoji = NULL;
        NEW.custom_status_expires_at = NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- RELATIONSHIP FUNCTIONS
-- =====================================================

CREATE OR REPLACE FUNCTION sync_bidirectional_relationship()
RETURNS TRIGGER AS $$
DECLARE
    reverse_type relationship_type;
    reverse_message TEXT;
BEGIN
    -- Prevent recursive calls
    IF pg_trigger_depth() > 1 THEN
        IF TG_OP = 'DELETE' THEN
            RETURN OLD;
        ELSE
            RETURN NEW;
        END IF;
    END IF;

    IF TG_OP = 'INSERT' THEN
        CASE NEW.type
            WHEN 'pending_outgoing' THEN
                reverse_type := 'pending_incoming';
                reverse_message := NEW.message;
            WHEN 'pending_incoming' THEN
                reverse_type := 'pending_outgoing';
                reverse_message := NEW.message;
            WHEN 'friend' THEN
                reverse_type := 'friend';
                reverse_message := NULL;
            WHEN 'blocked' THEN
                RETURN NEW;
            ELSE
                RETURN NEW;
        END CASE;

        INSERT INTO user_relationships (user_id, target_user_id, type, message)
        VALUES (NEW.target_user_id, NEW.user_id, reverse_type, reverse_message)
        ON CONFLICT (user_id, target_user_id) DO UPDATE
        SET type = reverse_type,
            message = reverse_message,
            updated_at = NOW();

    ELSIF TG_OP = 'UPDATE' THEN
        CASE NEW.type
            WHEN 'pending_outgoing' THEN
                reverse_type := 'pending_incoming';
                reverse_message := NEW.message;
            WHEN 'pending_incoming' THEN
                reverse_type := 'pending_outgoing';
                reverse_message := NEW.message;
            WHEN 'friend' THEN
                reverse_type := 'friend';
                reverse_message := NULL;
            WHEN 'blocked' THEN
                RETURN NEW;
            ELSE
                RETURN NEW;
        END CASE;

        UPDATE user_relationships
        SET type       = reverse_type,
            message    = reverse_message,
            updated_at = NOW()
        WHERE user_id = NEW.target_user_id
          AND target_user_id = NEW.user_id;

    ELSIF TG_OP = 'DELETE' THEN
        DELETE FROM user_relationships
        WHERE user_id = OLD.target_user_id
          AND target_user_id = OLD.user_id
          AND OLD.type != 'blocked';
        RETURN OLD;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_user_relationships_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

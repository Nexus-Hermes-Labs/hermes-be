CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION update_session_last_used()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_used_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION log_auth_events()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.last_login_at IS DISTINCT FROM NEW.last_login_at THEN
        INSERT INTO auth_audit_log (credential_id, event_type, event_description, ip_address)
        VALUES (NEW.id, 'login', 'User logged in', NEW.last_login_ip);
    END IF;

    IF OLD.password_hash IS DISTINCT FROM NEW.password_hash THEN
        INSERT INTO auth_audit_log (credential_id, event_type, event_description)
        VALUES (NEW.id, 'password_change', 'Password changed');
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

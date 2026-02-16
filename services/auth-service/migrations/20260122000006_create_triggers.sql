CREATE TRIGGER update_auth_credentials_updated_at
    BEFORE UPDATE ON auth_credentials
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER log_auth_credentials_events
    AFTER UPDATE ON auth_credentials
    FOR EACH ROW
    EXECUTE FUNCTION log_auth_events();

CREATE TRIGGER update_auth_sessions_last_used
    BEFORE UPDATE ON auth_sessions
    FOR EACH ROW
    WHEN (OLD.* IS DISTINCT FROM NEW.*)
    EXECUTE FUNCTION update_session_last_used();

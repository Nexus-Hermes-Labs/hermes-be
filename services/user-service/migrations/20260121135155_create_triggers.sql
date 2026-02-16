-- =====================================================
-- USER_PROFILES TRIGGERS
-- =====================================================

CREATE TRIGGER update_user_profiles_updated_at
    BEFORE UPDATE
    ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER create_user_profiles_privacy_settings
    AFTER INSERT
    ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION create_default_privacy_settings();

CREATE TRIGGER update_user_profiles_last_seen
    BEFORE UPDATE
    ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_last_seen_on_status_change();

CREATE TRIGGER clean_user_profiles_custom_status
    BEFORE UPDATE
    ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION clean_expired_custom_status();

-- =====================================================
-- USER_PRIVACY_SETTINGS TRIGGERS
-- =====================================================

CREATE TRIGGER update_user_privacy_settings_updated_at
    BEFORE UPDATE
    ON user_privacy_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- =====================================================
-- USER_RELATIONSHIPS TRIGGERS
-- =====================================================

CREATE TRIGGER trg_sync_bidirectional_relationship
    AFTER INSERT OR UPDATE OR DELETE
    ON user_relationships
    FOR EACH ROW
    EXECUTE FUNCTION sync_bidirectional_relationship();

CREATE TRIGGER trg_user_relationships_updated_at
    BEFORE UPDATE
    ON user_relationships
    FOR EACH ROW
    EXECUTE FUNCTION update_user_relationships_updated_at();

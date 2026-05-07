-- Track the IP that last refreshed a session, separate from the IP that
-- created it. `ip_address` keeps its original meaning ("session creation IP")
-- so we can spot geo-anomalies between creation and recent use.

ALTER TABLE auth_sessions
    ADD COLUMN last_used_ip INET;

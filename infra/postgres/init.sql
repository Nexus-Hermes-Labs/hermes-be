-- PostgreSQL initialization script
-- Creates a separate database for each microservice
-- Runs automatically on first container start (docker-entrypoint-initdb.d)

CREATE DATABASE hermes_auth;
CREATE DATABASE hermes_user;
CREATE DATABASE hermes_guild;
CREATE DATABASE hermes_channel;
CREATE DATABASE hermes_messaging;

GRANT ALL PRIVILEGES ON DATABASE hermes_auth TO hermes;
GRANT ALL PRIVILEGES ON DATABASE hermes_user TO hermes;
GRANT ALL PRIVILEGES ON DATABASE hermes_guild TO hermes;
GRANT ALL PRIVILEGES ON DATABASE hermes_channel TO hermes;
GRANT ALL PRIVILEGES ON DATABASE hermes_messaging TO hermes;

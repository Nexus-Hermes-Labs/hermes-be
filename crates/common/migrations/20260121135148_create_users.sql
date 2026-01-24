-- Migration: 20240101000000_create_users
-- Description: Create users table for authentication and user management
-- Service: Auth Service (primary), User Service (secondary)
-- Author: Bulut
-- Date: 2026-01-21

begin;

-- enable required extensions
create extension if not exists "uuid-ossp";

-- custom types
create type user_status as enum ('online', 'offline', 'idle', 'dnd');
create type user_role as enum ('user', 'moderator', 'admin');

-- users table
create table users (
    id uuid primary key default uuid_generate_v4(),
    
    -- authentication
    username varchar(32) not null unique
        check (username ~* '^[a-z0-9_-]+$' and length(username) >= 3),
    email varchar(255) not null unique
        check (email ~* '^[a-za-z0-9._%+-]+@[a-za-z0-9.-]+\.[a-za-z]{2,}$'),
    password_hash varchar(255) not null,
    
    -- profile
    display_name varchar(100) not null
        check (length(trim(display_name)) >= 1),
    avatar_url varchar(512)
        check (avatar_url is null or avatar_url ~* '^https?://'),
    bio text
        check (bio is null or length(bio) <= 500),
    
    -- status
    status user_status not null default 'offline',
    custom_status varchar(128),
    
    -- access control
    role user_role not null default 'user',
    is_active boolean not null default true,
    
    -- verification
    email_verified boolean not null default false,
    email_verification_token varchar(64),
    
    -- soft delete
    deleted_at timestamptz,
    
    -- timestamps
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

-- primary indexes
create index idx_users_email on users(email) where deleted_at is null;
create index idx_users_username on users(username) where deleted_at is null;
create index idx_users_status on users(status) where deleted_at is null;
create index idx_users_role on users(role);

-- verification index
create index idx_users_email_verification on users(email_verification_token)
    where email_verification_token is not null;

-- active users index
create index idx_users_active on users(is_active, status) 
    where deleted_at is null and is_active = true;

-- soft delete index
create index idx_users_deleted_at on users(deleted_at) where deleted_at is not null;

-- search index (optional, can be added later if needed)
create index idx_users_search on users using gin(
    to_tsvector('english', display_name || ' ' || username)
) where deleted_at is null;

-- function to auto-update updated_at
create or replace function update_updated_at_column()
returns trigger as $$
begin
    new.updated_at = now();
    return new;
end;
$$ language 'plpgsql';

-- function to validate email change
create or replace function validate_email_change()
returns trigger as $$
begin
    -- if email changed, reset verification
    if old.email is distinct from new.email then
        new.email_verified = false;
        new.email_verification_token = null;
    end if;
    return new;
end;
$$ language 'plpgsql';

-- triggers
create trigger update_users_updated_at 
    before update on users
    for each row
    execute function update_updated_at_column();

create trigger validate_users_email_change
    before update on users
    for each row
    execute function validate_email_change();

-- comments for documentation
comment on table users is 'user accounts and authentication data (mvp)';
comment on column users.id is 'unique user identifier (uuid v4)';
comment on column users.username is 'unique username for login (3-32 chars, lowercase alphanumeric)';
comment on column users.email is 'unique email for login and notifications';
comment on column users.password_hash is 'argon2id hashed password';
comment on column users.display_name is 'user-facing display name (min 1 char)';
comment on column users.status is 'current online status: online, offline, idle, dnd';
comment on column users.role is 'user role: user, moderator, admin';
comment on column users.is_active is 'account active status (soft disable)';
comment on column users.deleted_at is 'soft delete timestamp';

commit;

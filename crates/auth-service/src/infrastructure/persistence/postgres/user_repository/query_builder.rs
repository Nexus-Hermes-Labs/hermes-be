use crate::domain::user::filters::UserFilters;
use common::persistance::sql_utils::escape_like_pattern;
use sqlx::{Postgres, QueryBuilder};

pub struct UserQueryBuilder;

impl UserQueryBuilder {
    pub fn apply_filters<'a>(qb: &mut QueryBuilder<'a, Postgres>, filters: &'a UserFilters) {
        let mut separator = WhereClause::new();

        // Active/deleted filter (hardcoded SQL - safe)
        match filters.is_active {
            Some(true) => {
                separator.push(qb);
                qb.push("deleted_at IS NULL");
            }
            Some(false) => {
                separator.push(qb);
                qb.push("deleted_at IS NOT NULL");
            }
            None => {}
        }

        // Email filter (parameterized - safe)
        if let Some(ref email) = filters.email {
            separator.push(qb);
            qb.push("email ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(email)));
        }

        // Username filter (parameterized - safe)
        if let Some(ref username) = filters.username {
            separator.push(qb);
            qb.push("username ILIKE ")
                .push_bind(format!("%{}%", escape_like_pattern(username)));
        }

        // Role filter (enum - safe)
        if let Some(ref role) = filters.role {
            separator.push(qb);
            qb.push("role = ").push_bind(role.as_str());
        }

        if let Some(ref search) = filters.search {
            separator.push(qb);

            let pattern = format!("%{}%", escape_like_pattern(search));

            qb.push("(");
            qb.push("email ILIKE ").push_bind(pattern.clone());
            qb.push(" OR username ILIKE ").push_bind(pattern.clone());
            qb.push(" OR first_name ILIKE ").push_bind(pattern.clone());
            qb.push(" OR last_name ILIKE ").push_bind(pattern);
            qb.push(")");
        }
    }
}

struct WhereClause {
    first: bool,
}

impl WhereClause {
    fn new() -> Self {
        Self { first: true }
    }

    fn push(&mut self, qb: &mut QueryBuilder<Postgres>) {
        if self.first {
            qb.push(" WHERE ");
            self.first = false;
        } else {
            qb.push(" AND ");
        }
    }
}

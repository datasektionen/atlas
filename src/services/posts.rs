use crate::{
    dto::posts::EditPostDto,
    errors::{AppError, AppResult},
    guards::user::User,
    models::{Post, PostId, PostModel},
};

use super::update_if_changed;

pub async fn get_one<'x, X, P: PostModel>(id: i64, db: X) -> AppResult<Option<P>>
where
    X: sqlx::Executor<'x, Database = sqlx::Postgres>,
{
    let post = sqlx::query_as("SELECT * FROM post WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await?;

    Ok(post)
}

pub async fn require_one<'x, X, P: PostModel>(id: i64, db: X) -> AppResult<P>
where
    X: sqlx::Executor<'x, Database = sqlx::Postgres>,
{
    get_one(id, db)
        .await?
        .ok_or_else(|| AppError::NoSuchPost(id))
}

pub async fn create<'v, 'x, X>(dto: &EditPostDto<'v>, db: X, user: &User) -> AppResult<i64>
where
    X: sqlx::Acquire<'x, Database = sqlx::Postgres>,
{
    let mut txn = db.begin().await?;

    let mut query = sqlx::QueryBuilder::new(
        "INSERT INTO post (darkmode_hide, published, author, mandate, title_sv, title_en, content_sv, content_en",
    );

    // Only insert publish_time if it is set, otherwise use default
    if dto.publish_time.is_some() {
        query.push(", publish_time");
    }
    query.push(") VALUES (");

    let mut separated = query.separated(", ");
    separated.push_bind(dto.darkmode_hide);
    separated.push_bind(dto.publish.pressed());
    separated.push_bind(user.username());
    separated.push_bind(dto.mandate);
    separated.push_bind(dto.title_sv);
    separated.push_bind(dto.title_en);
    separated.push_bind(dto.content_sv);
    separated.push_bind(dto.content_en);

    if let Some(publish_time) = &dto.publish_time {
        separated.push_bind(publish_time);
    }
    separated.push_unseparated(") RETURNING id");

    let post_id: PostId = query.build_query_as().fetch_one(&mut *txn).await?;

    txn.commit().await?;

    Ok(post_id.id)
}

pub async fn update<'v, 'x, X>(id: i64, dto: &EditPostDto<'v>, db: X) -> AppResult<()>
where
    X: sqlx::Acquire<'x, Database = sqlx::Postgres>,
{
    let mut txn = db.begin().await?;

    let old: Post = require_one(id, &mut *txn).await?;

    let mut query = sqlx::QueryBuilder::new("UPDATE post SET");
    let mut changed = false;

    update_if_changed!(changed, query, darkmode_hide, old, dto; skip_deref);
    // XXX: This is horribly ugly, but works
    let string_mandate = dto.mandate.map(|s| s.0.to_string());
    update_if_changed!(internal; changed, query, mandate, old.mandate, string_mandate, string_mandate);
    update_if_changed!(changed, query, title_sv, old, dto);
    update_if_changed!(changed, query, title_en, old, dto);
    update_if_changed!(changed, query, content_sv, old, dto);
    update_if_changed!(changed, query, content_en, old, dto);

    // Special treatment for some props
    if dto.publish.pressed() && dto.publish_time != Some(old.publish_time) {
        if changed {
            query.push(", ");
        }
        query.push(" publish_time = ");
        if let Some(publish_time) = &dto.publish_time {
            query.push_bind(publish_time);
        } else {
            // Default value
            query.push("NOW()");
        }
    }

    if (old.published && dto.draft.pressed()) || (!old.published && dto.publish.pressed()) {
        // toggle publish status
        if changed {
            query.push(", ");
        }
        query.push(" published = ");
        query.push_bind(!old.published);
        changed = true;
    }

    if changed {
        // Always update edit_time
        query
            .push(", edit_time = NOW() WHERE id =")
            .push_bind(id)
            .build()
            .execute(&mut *txn)
            .await?;

        txn.commit().await?;
    }

    Ok(())
}

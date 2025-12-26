use crate::{
    dto::posts::EditPostDto,
    errors::{AppError, AppResult},
    guards::user::User,
    models::{PostId, PostModel},
};

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

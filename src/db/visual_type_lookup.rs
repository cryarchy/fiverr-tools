use sqlx::PgPool;

pub struct VisualTypeLookup {
    pool: PgPool,
}

impl VisualTypeLookup {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, value: &str) -> Result<i64, super::Error> {
        let insert_result = sqlx::query!(
            "
            INSERT INTO visual_type_lookup (
                value
            )
            VALUES ($1)
            RETURNING id
        ",
            value
        )
        .fetch_one(&self.pool)
        .await;

        if let Err(sqlx::Error::RowNotFound) = insert_result {
            return Err(super::Error::Unexpected(
                "expected record to be returned after successful execution of create ".to_owned(),
            ));
        }

        insert_result
            .map(|record| record.id)
            .map_err(super::Error::Sqlx)
    }

    pub async fn get_type_id(&self, value: &str) -> Result<Option<i64>, super::Error> {
        let fetch_result = sqlx::query!(
            "
                SELECT id
                FROM visual_type_lookup
                WHERE value = $1
                LIMIT 1;
            ",
            value
        )
        .fetch_one(&self.pool)
        .await;

        if let Err(sqlx::Error::RowNotFound) = fetch_result {
            return Ok(None);
        }

        fetch_result
            .map(|record| Some(record.id))
            .map_err(super::Error::Sqlx)
    }
}

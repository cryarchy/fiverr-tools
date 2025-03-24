use sqlx::PgPool;

pub struct GigPackageTypeLookup {
    pool: PgPool,
}

impl GigPackageTypeLookup {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, name: &str) -> Result<i64, super::Error> {
        let insert_result = sqlx::query!(
            "
            INSERT INTO gig_package_type_lookup (
                name
            )
            VALUES ($1)
            RETURNING id
        ",
            name
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

    pub async fn get_type_id(&self, name: &str) -> Result<Option<i64>, super::Error> {
        let fetch_result = sqlx::query!(
            "
                SELECT id
                FROM gig_package_type_lookup
                WHERE name = $1
                LIMIT 1;
            ",
            name
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

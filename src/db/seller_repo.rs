use sqlx::PgPool;

pub struct CreateParams {
    pub username: String,
    pub rating: String,
    pub level: String,
    pub reviews_count: i64,
    pub description: String,
}

pub struct SellerRepo {
    pool: PgPool,
}

impl SellerRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_id(&self, username: &str) -> Result<Option<i64>, super::Error> {
        let fetch_result = sqlx::query!(
            "
                SELECT id
                FROM seller
                WHERE username = $1
                LIMIT 1;
            ",
            username
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

    pub async fn create(&self, params: &CreateParams) -> Result<i64, super::Error> {
        let insert_result = sqlx::query!(
            "
            INSERT INTO seller (
                username,
                rating,
                level,
                reviews_count,
                description
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
        ",
            &params.username,
            &params.rating,
            params.level,
            params.reviews_count,
            &params.description
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
}

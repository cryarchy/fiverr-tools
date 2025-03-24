use sqlx::PgPool;

pub struct CreateParams {
    pub gig_id: i64,
    pub key: String,
    pub values: Vec<String>,
}

pub struct GigMetadataRepo {
    pool: PgPool,
}

impl GigMetadataRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, params: &CreateParams) -> Result<i64, super::Error> {
        let insert_result = sqlx::query!(
            "
            INSERT INTO gig_metadata (
                gig_id,
                key,
                values
            )
            VALUES ($1, $2, $3)
            RETURNING id
        ",
            &params.gig_id,
            &params.key,
            &params.values
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

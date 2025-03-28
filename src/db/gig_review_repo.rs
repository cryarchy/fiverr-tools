use sqlx::PgPool;

pub struct CreateParams {
    pub gig_id: i64,
    pub country: Option<String>,
    pub rating: f64,
    pub price_range_min: i64,
    pub price_range_max: i64,
    pub duration_value: i64,
    pub duration_unit: String,
    pub description: String,
}

pub struct GigReviewRepo {
    pool: PgPool,
}

impl GigReviewRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, params: &CreateParams) -> Result<i64, super::Error> {
        let insert_result = sqlx::query!(
            "
            INSERT INTO gig_review (
                gig_id,
                country,
                rating,
                price_range_min,
                price_range_max,
                duration_value,
                duration_unit,
                description
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
        ",
            params.gig_id,
            params.country,
            params.rating,
            params.price_range_min,
            params.price_range_max,
            params.duration_value,
            params.duration_unit,
            params.description,
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

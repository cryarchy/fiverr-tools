use sqlx::PgPool;

pub struct CreateParams {
    pub r#type: i64,
    pub price: f64,
    pub title: String,
    pub description: Option<String>,
    pub gig_id: i64,
    pub delivery_time: Option<String>,
}

pub struct GigPackageRepo {
    pool: PgPool,
}

impl GigPackageRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, params: &CreateParams) -> Result<i64, super::Error> {
        let insert_result = sqlx::query!(
            "
            INSERT INTO gig_package (
                type,
                price,
                title,
                description,
                gig_id,
                delivery_time
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
        ",
            &params.r#type,
            params.price,
            params.title,
            params.description,
            params.gig_id,
            params.delivery_time
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

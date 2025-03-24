use sqlx::PgPool;

pub struct CreateParams {
    pub gig_id: i64,
    pub question: String,
    pub answer: String,
}

pub struct GigFaqRepo {
    pool: PgPool,
}

impl GigFaqRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, params: &CreateParams) -> Result<i64, super::Error> {
        let insert_result = sqlx::query!(
            "
            INSERT INTO gig_faq (
                gig_id,
                question,
                answer
            )
            VALUES ($1, $2, $3)
            RETURNING id
        ",
            &params.gig_id,
            &params.question,
            params.answer
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

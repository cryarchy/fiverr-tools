use sqlx::PgPool;

pub struct CreateParams {
    pub package_id: i64,
    pub key: String,
    pub value: String,
}

pub struct GigPackageFeatureRepo {
    pool: PgPool,
}

impl GigPackageFeatureRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, params: &CreateParams) -> Result<i64, super::Error> {
        let insert_result = sqlx::query!(
            "
            INSERT INTO gig_package_feature (
                gig_package_id,
                key,
                value
            )
            VALUES ($1, $2, $3)
            RETURNING id
        ",
            &params.package_id,
            &params.key,
            &params.value
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

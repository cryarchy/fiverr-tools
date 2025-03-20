use sqlx::PgPool;

pub struct GigRepo {
    pool: PgPool,
}

impl GigRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn count_for_category(&self, category_id: i64) -> Result<i64, super::Error> {
        let record = sqlx::query!(
            "
                SELECT COUNT(*) as gig_count
                FROM gig
                WHERE category_id = $1
                GROUP BY category_id;
            ",
            category_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(super::Error::Sqlx)?;

        match record.gig_count {
            Some(gig_count) => Ok(gig_count),
            None => Err(super::Error::Unexpected(
                "expected gig_count to have a value".to_owned(),
            )),
        }
    }

    pub async fn delete_partially_scraped_gigs(&self) -> Result<u64, super::Error> {
        sqlx::query!(
            "
                DELETE FROM gig
                WHERE scrape_completed = false;
            "
        )
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected())
        .map_err(super::Error::Sqlx)
    }

    pub async fn get_page_of_last_scraped_gig(
        &self,
        category_id: i64,
    ) -> Result<Option<i64>, super::Error> {
        let fetch_result = sqlx::query!(
            "
                SELECT page
                FROM gig
                WHERE category_id = $1
                ORDER BY id DESC
                LIMIT 1;
            ",
            category_id
        )
        .fetch_one(&self.pool)
        .await;

        if let Err(sqlx::Error::RowNotFound) = fetch_result {
            return Ok(None);
        }

        fetch_result
            .map(|record| Some(record.page))
            .map_err(super::Error::Sqlx)
    }

    pub async fn exists_by_path(&self, path: &str) -> Result<bool, super::Error> {
        let fetch_result = sqlx::query!(
            "
                SELECT id
                FROM gig
                WHERE path = $1
                LIMIT 1;
            ",
            path
        )
        .fetch_one(&self.pool)
        .await;

        if let Err(sqlx::Error::RowNotFound) = fetch_result {
            return Ok(false);
        }

        fetch_result.map(|_| true).map_err(super::Error::Sqlx)
    }
}

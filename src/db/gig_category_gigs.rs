use sqlx::PgPool;

pub struct GigCategoryGigs {
    pool: PgPool,
}

impl GigCategoryGigs {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn least_gigs_count_for_categories(&self) -> Result<i64, super::Error> {
        let record = sqlx::query!(
            "
                SELECT COALESCE(gig_count, 0) as gig_count
                FROM gig_category gc
                LEFT JOIN (
                    SELECT category_id, COUNT(*) as gig_count
                    FROM gig
                    GROUP BY category_id
                ) g ON gc.id = g.category_id
                WHERE gc.scrape_gigs = true
                ORDER BY gig_count ASC
                LIMIT 1;
            "
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
}

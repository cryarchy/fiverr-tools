use sqlx::PgPool;

pub struct CreateParams {
    pub path: String,
    pub title: String,
    pub rating: String,
    pub reviews_count: i64,
    pub description: String,
    pub page: i64,
    pub seller_id: i64,
    pub category_id: i64,
}

pub struct GigRepo {
    pool: PgPool,
}

impl GigRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn set_scrape_completed(&self, id: i64) -> Result<bool, super::Error> {
        let update_result = sqlx::query!(
            "
                UPDATE gig
                SET scrape_completed = true
                WHERE id = $1
            ",
            id
        )
        .execute(&self.pool)
        .await
        .map_err(super::Error::Sqlx)?;

        Ok(update_result.rows_affected() == 1)
    }

    pub async fn create(&self, params: &CreateParams) -> Result<i64, super::Error> {
        let insert_result = sqlx::query!(
            "
                INSERT INTO gig (
                    path,
                    title,
                    rating,
                    reviews_count,
                    description,
                    page,
                    seller_id,
                    category_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id
            ",
            &params.path,
            &params.title,
            &params.rating,
            params.reviews_count,
            &params.description,
            params.page,
            params.seller_id,
            params.category_id
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

    pub async fn count_for_category(&self, category_id: i64) -> Result<i64, super::Error> {
        let count_result = sqlx::query!(
            "
                SELECT COUNT(*) as gig_count
                FROM gig
                WHERE category_id = $1
                GROUP BY category_id;
            ",
            category_id
        )
        .fetch_one(&self.pool)
        .await;

        if let Err(sqlx::Error::RowNotFound) = count_result {
            return Ok(0);
        }

        match count_result.map_err(super::Error::Sqlx)?.gig_count {
            Some(gig_count) => Ok(gig_count),
            None => Err(super::Error::Unexpected(
                "expected gig_count to have a value".to_owned(),
            )),
        }
    }

    pub async fn delete_partially_scraped_gigs(&self) -> Result<u64, super::Error> {
        let fetch_result = sqlx::query!(
            "
                SELECT id
                FROM gig
                WHERE scrape_completed = false;
            "
        )
        .fetch_all(&self.pool)
        .await;

        if let Err(sqlx::Error::RowNotFound) = fetch_result {
            return Ok(0);
        }

        let partial_gigs_ids = fetch_result?.into_iter().map(|r| r.id).collect::<Vec<_>>();

        let fetch_result = sqlx::query!(
            "SELECT id FROM gig_package WHERE gig_id IN (SELECT unnest($1::integer[]))",
            &partial_gigs_ids as &Vec<i64>
        )
        .fetch_all(&self.pool)
        .await;

        let gig_packages_ids = match fetch_result {
            Ok(records) => records.into_iter().map(|r| r.id).collect::<Vec<_>>(),
            Err(sqlx::Error::RowNotFound) => Vec::new(),
            Err(e) => return Err(e.into()),
        };

        // gig_package_feature
        sqlx::query!(
            "DELETE FROM gig_package_feature WHERE gig_package_id IN (SELECT unnest($1::integer[]))",
            &gig_packages_ids as &Vec<i64>
        )
        .execute(&self.pool)
        .await?;

        // gig_package
        sqlx::query!(
            "DELETE FROM gig_package WHERE id IN (SELECT unnest($1::integer[]))",
            gig_packages_ids as Vec<i64>
        )
        .execute(&self.pool)
        .await?;

        // gig_metadata
        sqlx::query!(
            "DELETE FROM gig_metadata WHERE gig_id IN (SELECT unnest($1::integer[]))",
            &partial_gigs_ids as &Vec<i64>
        )
        .execute(&self.pool)
        .await?;

        // gig_visual
        sqlx::query!(
            "DELETE FROM gig_visual WHERE gig_id IN (SELECT unnest($1::integer[]))",
            &partial_gigs_ids as &Vec<i64>
        )
        .execute(&self.pool)
        .await?;

        // gig_faq
        sqlx::query!(
            "DELETE FROM gig_faq WHERE gig_id IN (SELECT unnest($1::integer[]))",
            &partial_gigs_ids as &Vec<i64>
        )
        .execute(&self.pool)
        .await?;

        // gig_review
        sqlx::query!(
            "DELETE FROM gig_review WHERE gig_id IN (SELECT unnest($1::integer[]))",
            &partial_gigs_ids as &Vec<i64>
        )
        .execute(&self.pool)
        .await?;

        // gig
        sqlx::query!(
            "DELETE FROM gig WHERE id IN (SELECT unnest($1::integer[]))",
            &partial_gigs_ids as &Vec<i64>
        )
        .execute(&self.pool)
        .await?;

        Ok(partial_gigs_ids.len() as u64)
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

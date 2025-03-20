use sqlx::PgPool;

pub struct CreateParams {
    pub path: String,
    pub name: String,
    pub sub_group_name: String,
    pub main_group_name: String,
}

pub struct ScrapeGigsOutput {
    pub scrape_gigs: bool,
    pub name: String,
    pub id: i64,
}

pub struct GigCategoryRepo {
    pool: PgPool,
}

impl GigCategoryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_scrape_gigs(
        &self,
        url: &str,
    ) -> Result<Option<ScrapeGigsOutput>, super::Error> {
        let fetch_result = sqlx::query!(
            "
                SELECT id, scrape_gigs, name
                FROM gig_category
                WHERE path = $1
            ",
            url
        )
        .fetch_one(&self.pool)
        .await;

        if let Err(sqlx::Error::RowNotFound) = fetch_result {
            return Ok(None);
        }

        let gig_category_record = fetch_result.map_err(super::Error::Sqlx)?;

        Ok(Some(ScrapeGigsOutput {
            id: gig_category_record.id,
            name: gig_category_record.name,
            scrape_gigs: gig_category_record.scrape_gigs,
        }))
    }

    pub async fn create(&self, params: &CreateParams) -> Result<i64, super::Error> {
        let insert_result = sqlx::query!(
            "
                INSERT INTO gig_category (
                    path,
                    name,
                    sub_group_name,
                    main_group_name
                )
                VALUES ($1, $2, $3, $4)
                RETURNING id
            ",
            &params.path,
            &params.name,
            &params.sub_group_name,
            &params.main_group_name
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

    pub async fn record_with_least_gigs(&self) -> Result<i64, super::Error> {
        let insert_result = sqlx::query!(
            "
                SELECT gc.id, gc.path, gc.name, COALESCE(gig_count, 0) as gig_count
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

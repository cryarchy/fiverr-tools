-- 1. Create the seller table.
CREATE TABLE seller (
    id BIGSERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    rating TEXT NOT NULL,
    level TEXT NOT NULL,
    reviews_count BIGINT NOT NULL,
    description TEXT NOT NULL
);

-- 2. Create the gig_category table.
CREATE TABLE gig_category (
    id BIGSERIAL PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    scrape_gigs BOOLEAN NOT NULL DEFAULT FALSE,
    sub_group_name TEXT NOT NULL,
    main_group_name TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS path_idx ON gig_category (path);

-- 3. Create the visual_type_lookup table.
CREATE TABLE visual_type_lookup (
    id BIGSERIAL PRIMARY KEY,
    value TEXT NOT NULL UNIQUE
);

-- 4. Create the gig_package_type_lookup table to support gig_package.type as a foreign key.
CREATE TABLE gig_package_type_lookup (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

-- 5. Create the gig table.
CREATE TABLE gig (
    id BIGSERIAL PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    rating TEXT NOT NULL,
    reviews_count BIGINT NOT NULL,
    description TEXT NOT NULL,
    scrape_completed BOOLEAN NOT NULL DEFAULT FALSE,
    page BIGINT NOT NULL,
    seller_id BIGINT NOT NULL,
    category_id BIGINT NOT NULL,
    FOREIGN KEY (seller_id) REFERENCES seller(id),
    FOREIGN KEY (category_id) REFERENCES gig_category(id)
);

-- 6. Create the gig_metadata table.
CREATE TABLE gig_metadata (
    id BIGSERIAL PRIMARY KEY,
    key TEXT NOT NULL,
    values TEXT[] NOT NULL,
    gig_id BIGINT NOT NULL,
    FOREIGN KEY (gig_id) REFERENCES gig(id),
    UNIQUE (gig_id, key)
);

-- 7. Create the gig_visual table.
CREATE TABLE gig_visual (
    id BIGSERIAL PRIMARY KEY,
    gig_id BIGINT NOT NULL,
    url TEXT NOT NULL,
    file_path TEXT,
    visual_type BIGINT NOT NULL,
    FOREIGN KEY (gig_id) REFERENCES gig(id),
    FOREIGN KEY (visual_type) REFERENCES visual_type_lookup(id)
);

-- 8. Create the seller_stat table.
CREATE TABLE seller_stat (
    id BIGSERIAL PRIMARY KEY,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    seller_id BIGINT NOT NULL,
    FOREIGN KEY (seller_id) REFERENCES seller(id),
    UNIQUE (seller_id, key)
);

-- 9. Create the gig_faq table.
CREATE TABLE gig_faq (
    id BIGSERIAL PRIMARY KEY,
    gig_id BIGINT NOT NULL,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    FOREIGN KEY (gig_id) REFERENCES gig(id)
);

-- 10. Create the gig_package table.
CREATE TABLE gig_package (
    id BIGSERIAL PRIMARY KEY,
    type BIGINT NOT NULL,  -- references gig_package_type_lookup
    price DOUBLE PRECISION NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    gig_id BIGINT NOT NULL,
    delivery_time TEXT,
    FOREIGN KEY (gig_id) REFERENCES gig(id),
    FOREIGN KEY (type) REFERENCES gig_package_type_lookup(id)
);

-- 11. Create the gig_package_feature table.
CREATE TABLE gig_package_feature (
    id BIGSERIAL PRIMARY KEY,
    gig_package_id BIGINT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    FOREIGN KEY (gig_package_id) REFERENCES gig_package(id),
    UNIQUE (gig_package_id, key)
);

-- 12. Create the gig_review table.
CREATE TABLE gig_review (
    id BIGSERIAL PRIMARY KEY,
    gig_id BIGINT NOT NULL,
    country TEXT,
    rating DOUBLE PRECISION NOT NULL,
    price_range_min BIGINT NOT NULL,
    price_range_max BIGINT NOT NULL,
    duration_value BIGINT NOT NULL,
    duration_unit TEXT NOT NULL,
    description TEXT NOT NULL,
    FOREIGN KEY (gig_id) REFERENCES gig(id),
    CHECK (price_range_min <= price_range_max)
);

-- Create the gig table.
CREATE TABLE gigs (
    id VARCHAR(100) PRIMARY KEY NOT NULL,
    url TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    page BIGINT NOT NULL
);

-- Create the visuals table.
CREATE TABLE visuals (
    id VARCHAR(100) PRIMARY KEY NOT NULL,
    gig_id BIGINT NOT NULL,
    file_path TEXT,
    visual_type TEXT NOT NULL,
    FOREIGN KEY (gig_id) REFERENCES gigs(id) ON DELETE CASCADE
);
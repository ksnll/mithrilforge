DROP TABLE IF EXISTS email_campaigns;
DROP TABLE IF EXISTS contacts;
DROP TABLE IF EXISTS websites;

CREATE TABLE IF NOT EXISTS websites (
    website_id BIGSERIAL PRIMARY KEY,
    source_address VARCHAR(255) UNIQUE NOT NULL,
    generated_website_link VARCHAR(255),
    generated_website_name VARCHAR(255),
    contact_email VARCHAR(255),
    contact_name VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS email_campaigns (
    email_campaign_id BIGSERIAL PRIMARY KEY,
    body VARCHAR NOT NULL,
    subject VARCHAR NOT NULL,
    sent_at TIMESTAMPTZ,
    opened_at TIMESTAMPTZ,
    clicked_at TIMESTAMPTZ,
    tracking_token UUID DEFAULT GEN_RANDOM_UUID() NOT NULL,
    website_id BIGINT NOT NULL REFERENCES websites (
        website_id
    ) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


CREATE UNIQUE INDEX email_campaigns_tracking_token_idx ON email_campaigns (
    tracking_token
);
CREATE INDEX email_campaigns_website_id_idx ON email_campaigns (website_id);

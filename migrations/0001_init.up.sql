CREATE DOMAIN username AS TEXT CHECK (VALUE ~ '^[a-z0-9]{2,}$');
CREATE DOMAIN groupkey AS TEXT CHECK (VALUE ~ '^[a-z0-9]+(-[a-z0-9]+)*@[-a-z0-9]+\.[a-z]+$');
CREATE DOMAIN color AS CHAR(7) CHECK (VALUE ~ '^#[0-9a-f]{6}$');

CREATE TABLE post (
    -- incremental IDs instead of UUIDs to make URL:s prettier without having
    -- to generate slugs, since many posts are probably titled similarly
    id              BIGINT      NOT NULL    GENERATED ALWAYS AS IDENTITY,
    darkmode_hide   BOOLEAN     NOT NULL    DEFAULT FALSE,
    published       BOOLEAN     NOT NULL    DEFAULT FALSE,
    publish_time    TIMESTAMPTZ NOT NULL    DEFAULT NOW(),
    edit_time       TIMESTAMPTZ NOT NULL    DEFAULT NOW(),
    author          USERNAME    NOT NULL,
    mandate         GROUPKEY    NULL,
    title_sv        TEXT        NOT NULL    CHECK (title_sv <> ''),
    title_en        TEXT        NOT NULL    CHECK (title_en <> ''),
    content_sv      TEXT        NOT NULL    CHECK (title_sv <> ''),
    content_en      TEXT        NOT NULL    CHECK (title_en <> ''),
    banner          TEXT        NULL,

    PRIMARY KEY (id)
);

COMMENT ON COLUMN post.publish_time IS 'Ignored when published is false';
COMMENT ON COLUMN post.banner IS 'URL to banner image';

CREATE TABLE post_event (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id         BIGINT      NOT NULL,
    start_time      TIMESTAMPTZ NOT NULL,
    end_time        TIMESTAMPTZ NOT NULL,
    title_sv        TEXT        NOT NULL    CHECK (title_sv <> ''),
    title_en        TEXT        NOT NULL    CHECK (title_en <> ''),
    description_sv  TEXT        NULL        CHECK (description_sv <> ''),
    description_en  TEXT        NULL        CHECK (description_en <> ''),
    location        TEXT        NOT NULL    CHECK (location <> ''),

    FOREIGN KEY (post_id) REFERENCES post (id) ON DELETE CASCADE,
    CHECK (end_time >= start_time)
);

COMMENT ON COLUMN post_event.description_sv IS 'When NULL, refer back to content of connected post.';
COMMENT ON COLUMN post_event.description_en IS 'When NULL, refer back to content of connected post.';

CREATE TABLE tag (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    restricted      BOOLEAN     NOT NULL    DEFAULT FALSE,
    -- An emoji to use as an icon
    icon            CHAR        NULL        CHECK (icon <> ''),
    color           COLOR       NOT NULL    DEFAULT '#ffffff',
    tag             TEXT        NOT NULL    CHECK (tag <> '')
);

CREATE TABLE post_tag (
    post_id         BIGINT      NOT NULL,
    tag_id          UUID        NOT NULL,

    PRIMARY KEY (post_id, tag_id),
    FOREIGN KEY (post_id) REFERENCES post (id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tag (id) ON DELETE CASCADE
);

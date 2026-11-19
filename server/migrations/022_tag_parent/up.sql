CREATE TABLE tag_parent (
    tag_id BIGINT PRIMARY KEY REFERENCES tag (id) ON DELETE CASCADE,
    parent_id BIGINT REFERENCES tag (id) ON DELETE SET NULL,
    CHECK (tag_id <> parent_id)
);

CREATE INDEX tag_parent_parent_id_idx ON tag_parent (parent_id);

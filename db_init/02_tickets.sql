CREATE TABLE tickets (
    visible_id VARCHAR(25) PRIMARY KEY,
    ordered_by VARCHAR(40),
    CONSTRAINT fk_order
        FOREIGN KEY(ordered_by)
            REFERENCES orders(uuid)
            ON DELETE SET NULL
);

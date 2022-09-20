CREATE TABLE warnings (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    model_id INTEGER REFERENCES models(id) ON DELETE SET NULL,
    resolved_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    note TEXT,
    admin_note TEXT,
    created TIMESTAMP NOT NULL,
    updated TIMESTAMP NOT NULL
);

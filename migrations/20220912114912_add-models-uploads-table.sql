CREATE TABLE uploads (
    id SERIAL PRIMARY KEY,
    model_id INTEGER REFERENCES models(id) NOT NULL,
    filepath VARCHAR NOT NULL,
    created TIMESTAMP NOT NULL
);

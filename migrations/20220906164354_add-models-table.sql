CREATE TABLE models (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    description TEXT,
    duration FLOAT NOT NULL,
    height FLOAT NOT NULL,
    weight FLOAT NOT NULL,
    printer VARCHAR,
    material VARCHAR,
    author_id INTEGER REFERENCES users(id) NOT NULL,
    created TIMESTAMP NOT NULL,
    updated TIMESTAMP NOT NULL
);

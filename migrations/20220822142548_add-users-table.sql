create table users (
    id serial unique,
    email varchar(100) unique not null,
    password varchar(100) not null,
    is_staff boolean default false
);

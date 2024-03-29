-- migrate:up
create table users (
    id SERIAL PRIMARY KEY,
    image varchar(255),
    slug varchar(255) unique not null,
    email varchar(255) unique not null,
    first_name varchar(255),
    last_name varchar(255),
    password text not null,
    create_at timestamp with time zone not null,
    update_at timestamp with time zone,
    last_login timestamp with time zone
);

-- migrate:down
drop table users;


-- migrate:up
create table users (
    id VARCHAR(255) NOT NULL PRIMARY KEY,
    image varchar(255),
    username varchar(255) unique not null,
    email varchar(255) unique not null,
    first_name varchar(255),
    last_name varchar(255),
    password text not null,
    is_active BOOLEAN DEFAULT FALSE,
    create_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP not null,
    active_at timestamp with time zone,
    update_at timestamp with time zone,
    last_login timestamp with time zone
);

-- migrate:down
drop table users;


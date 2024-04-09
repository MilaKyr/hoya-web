CREATE DATABASE api;
\c api;

CREATE TABLE Product (
    id SERIAL PRIMARY KEY,
    name VARCHAR(256) NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE Shop (
    id SERIAL PRIMARY KEY,
    name VARCHAR(256) NOT NULL,
    url VARCHAR(256) NOT NULL,
    logo VARCHAR(512) NOT NULL,
    last_parsed TIMESTAMP
);

CREATE TABLE ShopPosition (
    id SERIAL PRIMARY KEY,
    product_id INT NOT NULL,
    shop_id  INT NOT NULL,
    image VARCHAR(512),
    price DECIMAL(6, 2) NOT NULL,
    url VARCHAR(256) NOT NULL,
    FOREIGN KEY (product_id) REFERENCES Product(id) ON DELETE CASCADE,
    FOREIGN KEY (shop_id) REFERENCES Shop(id) ON DELETE CASCADE
);

CREATE TABLE HistoricPrice
(
    id SERIAL PRIMARY KEY,
    product_id INT NOT NULL,
    date DATE NOT NULL,
    avg_price DECIMAL(6, 3),
    FOREIGN KEY (product_id) REFERENCES Product(id) ON DELETE CASCADE
);

CREATE TABLE Contacts
(
    id SERIAL PRIMARY KEY,
    email VARCHAR(256) NOT NULL UNIQUE
);

CREATE TABLE Messages
(
    id SERIAL PRIMARY KEY,
    email_id INT NOT NULL,
    date DATE NOT NULL,
    content TEXT NOT NULL,
    FOREIGN KEY (email_id) REFERENCES Contacts(id) ON DELETE CASCADE
);

CREATE TABLE Alerts
(
    id SERIAL PRIMARY KEY,
    product_id INT NOT NULL,
    email_id INT NOT NULL,
    created_date TIMESTAMP NOT NULL,
    alerted_date TIMESTAMP,
    delete_after BOOL DEFAULT FALSE,
    price_threshold DECIMAL(6, 3) NOT NULL,
    FOREIGN KEY (product_id) REFERENCES Product(id) ON DELETE CASCADE,
    FOREIGN KEY (email_id) REFERENCES Contacts(id) ON DELETE CASCADE
);


CREATE TABLE ParsingLookup (
    id SERIAL PRIMARY KEY,
    shop_id INT NOT NULL,
    max_page INT NOT NULL,
    product_table TEXT NOT NULL,
    product TEXT NOT NULL,
    name TEXT NOT NULL,
    price TEXT NOT NULL,
    url TEXT NOT NULL,
    FOREIGN KEY (shop_id) REFERENCES Shop(id) ON DELETE CASCADE
);

CREATE TABLE ShopParsingRules
(
    id SERIAL PRIMARY KEY,
    shop_id INT NOT NULL,
    url VARCHAR(256) NOT NULL,
    lookup_id INT NOT NULL,
    look_for_href BOOL DEFAULT FALSE,
    sleep_timeout_sec INT,
    FOREIGN KEY (lookup_id) REFERENCES ParsingLookup(id) ON DELETE CASCADE,
    FOREIGN KEY (shop_id) REFERENCES Shop(id) ON DELETE CASCADE
);

CREATE TABLE ParsingCategory
(
    id SERIAL PRIMARY KEY,
    shop_id INT NOT NULL,
    category VARCHAR(256) NOT NULL,
    FOREIGN KEY (shop_id) REFERENCES Shop(id) ON DELETE CASCADE
);

CREATE TABLE ProxySources
(
    id SERIAL PRIMARY KEY,
    source TEXT NOT NULL

);

CREATE TABLE ProxyParsingRules
(
    id SERIAL PRIMARY KEY,
    source_id INT NOT NULL,
    table_name TEXT NOT NULL,
    head TEXT NOT NULL,
    row TEXT NOT NULL,
    data TEXT NOT NULL,
    FOREIGN KEY (source_id) REFERENCES ProxySources(id) ON DELETE CASCADE

);

CREATE TABLE Proxy
(
    id SERIAL PRIMARY KEY,
    url TEXT NOT NULL
);




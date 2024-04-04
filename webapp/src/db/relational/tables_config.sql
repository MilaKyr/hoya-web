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
    logo VARCHAR(512) NOT NULL
);

CREATE TABLE ShopPosition (
    id SERIAL PRIMARY KEY,
    product_id INT NOT NULL,
    shop_id  INT NOT NULL,
    image VARCHAR(512),
    price DECIMAL(6, 2),
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

-- Add up migration script here
CREATE TABLE product (
    column_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    price REAL NOT NULL,
    quantity INTEGER NOT NULL
);

CREATE TABLE sale (
    id INTEGER PRIMARY KEY AUTOINCREMENT ,
    date DATETIME NOT NULL,
    price REAL NOT NULL,
    product_id INTEGER NOT NULL REFERENCES product(column_id)
);
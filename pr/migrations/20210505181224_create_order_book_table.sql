CREATE TABLE order_book (
   id SERIAL PRIMARY KEY,
   figi TEXT NOT NULL,
   received TIMESTAMP NOT NULL,
   content JSONB
);

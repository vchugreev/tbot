CREATE TABLE trade (
   id SERIAL PRIMARY KEY,
   figi TEXT NOT NULL,
   received TIMESTAMP NOT NULL,
   content JSONB
);

-- Add migration script here
ALTER TABLE orders
ADD COLUMN status TEXT NOT NULL DEFAULT 'pending'
CHECK (
    status IN ('pending', 'submitted', 'confirmed', 'failed')
);
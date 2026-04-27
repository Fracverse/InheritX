-- Add status column to admins table
ALTER TABLE admins ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'active'; -- active, locked, suspended

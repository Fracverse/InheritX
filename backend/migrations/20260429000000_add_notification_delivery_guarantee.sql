-- Add notification delivery guarantee columns
-- Tracks delivery status and retry attempts for notifications

ALTER TABLE notifications ADD COLUMN IF NOT EXISTS delivery_status VARCHAR(20) DEFAULT 'pending';
ALTER TABLE notifications ADD COLUMN IF NOT EXISTS delivery_attempts INTEGER DEFAULT 0;

-- Create index for finding undelivered notifications
CREATE INDEX IF NOT EXISTS idx_notifications_delivery_status ON notifications(delivery_status);

-- Create index for retrying failed notifications by creation time
CREATE INDEX IF NOT EXISTS idx_notifications_pending ON notifications(created_at) WHERE delivery_status IN ('pending', 'retrying');

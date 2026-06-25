CREATE TYPE kyc_status AS ENUM ('pending', 'submitted', 'approved', 'rejected');
CREATE TYPE payout_type AS ENUM ('crypto', 'fiat');
CREATE TYPE payout_status AS ENUM ('pending', 'processing', 'completed', 'failed');

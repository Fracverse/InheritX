-- Migration: Add plans and plan_logs tables

CREATE TABLE plans (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    amount NUMERIC(20, 6) NOT NULL,
    fee NUMERIC(20, 6) NOT NULL,
    net_amount NUMERIC(20, 6) NOT NULL,
    status VARCHAR(32) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE plan_logs (
    id SERIAL PRIMARY KEY,
    plan_id INTEGER NOT NULL REFERENCES plans(id),
    action VARCHAR(64) NOT NULL,
    performed_by INTEGER NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW()
);

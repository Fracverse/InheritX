DROP TRIGGER IF EXISTS beneficiaries_allocation_total_check ON beneficiaries;
DROP FUNCTION IF EXISTS check_plan_allocation_total();

DROP TABLE IF EXISTS payouts;
DROP TABLE IF EXISTS beneficiaries;
DROP TABLE IF EXISTS plans;
DROP TABLE IF EXISTS users;

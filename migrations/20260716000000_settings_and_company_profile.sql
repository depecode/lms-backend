-- Migration to create system_settings and company_profile tables
CREATE TABLE IF NOT EXISTS system_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(100) UNIQUE NOT NULL,
    value TEXT NOT NULL,
    type VARCHAR(50) NOT NULL,
    category VARCHAR(100) NOT NULL,
    description TEXT,
    editable BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS company_profile (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    registration_number VARCHAR(100) NOT NULL,
    industry VARCHAR(100) NOT NULL,
    country VARCHAR(100) NOT NULL,
    website VARCHAR(255),
    logo VARCHAR(255),
    phone VARCHAR(50) NOT NULL,
    email VARCHAR(150) NOT NULL,
    address TEXT NOT NULL,
    city VARCHAR(100) NOT NULL,
    state VARCHAR(100),
    zip_code VARCHAR(50),
    tax_id VARCHAR(100),
    license_number VARCHAR(100),
    license_expiry_date DATE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Seed System Settings
INSERT INTO system_settings (key, value, type, category, description, editable) VALUES
('company_name', 'LMS Pro Africa Ltd', 'string', 'General', 'Official name of the lending company', true),
('currency', 'NGN', 'string', 'General', 'Base operational currency code (e.g. NGN, UGX, USD)', true),
('timezone', 'Africa/Lagos', 'string', 'General', 'System default timezone for operations and scheduling', true),
('interest_method', 'Declining Balance', 'string', 'Loan', 'Default interest rate methodology applied to loan products', true),
('default_interest_rate', '15', 'number', 'Loan', 'Base default loan annual interest rate percentage', true),
('min_loan_term', '1', 'number', 'Loan', 'Minimum loan term duration allowed in months', true),
('max_loan_term', '36', 'number', 'Loan', 'Maximum loan term duration allowed in months', true),
('enable_collateral', 'true', 'boolean', 'Loan', 'Require collateral registration checks for active loans', true),
('min_opening_balance', '2000', 'number', 'Savings', 'Minimum savings account opening and activation balance (₦)', true),
('annual_interest_rate', '4', 'number', 'Savings', 'Annual savings interest yield rate percentage', true),
('withdrawal_fee', '100', 'number', 'Savings', 'Flat processing fee charged on savings withdrawals (₦)', true),
('send_email_notifications', 'true', 'boolean', 'Notification', 'Enable automated emails for repayments, approvals, and reminders', true),
('send_sms_notifications', 'false', 'boolean', 'Notification', 'Enable automatic SMS delivery for transaction alerts', true)
ON CONFLICT (key) DO NOTHING;

-- Seed Company Profile
INSERT INTO company_profile (id, name, registration_number, industry, country, website, phone, email, address, city, state, zip_code, tax_id, license_number, license_expiry_date) VALUES
('11111111-1111-1111-1111-111111111111', 'LMS Pro Africa Ltd', 'RC-1298471', 'Financial Services', 'Nigeria', 'www.lmsproafrica.com', '+234 803 123 4567', 'info@lmsproafrica.com', '12 Grace Plaza, Lekki Phase 1', 'Lagos', 'Lagos', '100001', 'T-928471-X', 'CBN-MFB-2023-098', '2028-12-31')
ON CONFLICT (id) DO NOTHING;

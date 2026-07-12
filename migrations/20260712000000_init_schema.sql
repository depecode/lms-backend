-- Enable UUID Extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =========================================================================
-- 1. UTILITY FUNCTIONS & TRIGGERS
-- =========================================================================

-- Automated updated_at timestamp function
CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- =========================================================================
-- 2. USER MANAGEMENT
-- =========================================================================

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL CHECK (role IN ('Admin', 'Loan Officer', 'Manager', 'Auditor', 'Viewer')),
    status VARCHAR(50) NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive', 'Suspended')),
    department VARCHAR(100),
    phone VARCHAR(50),
    avatar VARCHAR(255),
    last_login TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_users_modtime ON users;
CREATE TRIGGER update_users_modtime
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- =========================================================================
-- 3. BORROWER MANAGEMENT
-- =========================================================================

CREATE TABLE IF NOT EXISTS borrowers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    phone VARCHAR(50) UNIQUE NOT NULL,
    id_number VARCHAR(100) UNIQUE NOT NULL,
    date_of_birth DATE NOT NULL,
    address TEXT NOT NULL,
    city VARCHAR(100) NOT NULL,
    country VARCHAR(100) NOT NULL,
    kyc_status VARCHAR(50) NOT NULL DEFAULT 'pending' CHECK (kyc_status IN ('pending', 'approved', 'rejected', 'review')),
    status VARCHAR(50) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'blacklisted')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_borrowers_modtime ON borrowers;
CREATE TRIGGER update_borrowers_modtime
    BEFORE UPDATE ON borrowers
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- Borrower Documents Registry
CREATE TABLE IF NOT EXISTS borrower_documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    borrower_id UUID REFERENCES borrowers(id) ON DELETE CASCADE NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('id_card', 'passport', 'driving_license', 'birth_certificate', 'proof_of_address')),
    file_name VARCHAR(255) NOT NULL,
    file_url VARCHAR(255) NOT NULL,
    uploaded_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    expiry_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'unverified' CHECK (status IN ('verified', 'unverified', 'expired'))
);

-- Borrower Groups Table
CREATE TABLE IF NOT EXISTS borrower_groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    group_code VARCHAR(50) UNIQUE,
    description TEXT,
    type VARCHAR(50) CHECK (type IN ('Family', 'Business', 'Community', 'Peer Group', 'Other')),
    primary_borrower_id UUID REFERENCES borrowers(id) ON DELETE SET NULL,
    primary_contact VARCHAR(100),
    contact_email VARCHAR(255),
    contact_phone VARCHAR(50),
    leader_name VARCHAR(100),
    leader_phone VARCHAR(50),
    location VARCHAR(150),
    country VARCHAR(100),
    formation_date DATE,
    guarantee_percentage NUMERIC(5, 2) DEFAULT 0.00 CHECK (guarantee_percentage >= 0 AND guarantee_percentage <= 100),
    expected_member_count INT DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_borrower_groups_modtime ON borrower_groups;
CREATE TRIGGER update_borrower_groups_modtime
    BEFORE UPDATE ON borrower_groups
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- Joint borrower group members mapping table
CREATE TABLE IF NOT EXISTS borrower_group_members (
    group_id UUID REFERENCES borrower_groups(id) ON DELETE CASCADE,
    borrower_id UUID REFERENCES borrowers(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'Secondary' CHECK (role IN ('Primary', 'Secondary', 'Guarantor')),
    join_date DATE DEFAULT CURRENT_DATE,
    PRIMARY KEY (group_id, borrower_id)
);

DROP TABLE IF EXISTS guarantors CASCADE;

CREATE TABLE guarantors (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    borrower_id UUID REFERENCES borrowers(id) ON DELETE CASCADE NOT NULL,
    loan_id UUID REFERENCES loans(id) ON DELETE CASCADE,
    name VARCHAR(150) NOT NULL,
    relationship VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(50) NOT NULL,
    address TEXT,
    id_number VARCHAR(100),
    guarantee_amount NUMERIC(15, 2) NOT NULL DEFAULT 0.00 CHECK (guarantee_amount >= 0),
    liability_type VARCHAR(50) DEFAULT 'Joint and Several',
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    employment_status VARCHAR(100),
    income NUMERIC(15, 2),
    net_worth NUMERIC(15, 2),
    signature_date TIMESTAMP WITH TIME ZONE,
    signature_evidence VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_guarantors_modtime ON guarantors;
CREATE TRIGGER update_guarantors_modtime
    BEFORE UPDATE ON guarantors
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- =========================================================================
-- 4. LOAN & PRODUCT ENGINE
-- =========================================================================

CREATE TABLE IF NOT EXISTS loan_products (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    type VARCHAR(50) NOT NULL CHECK (type IN ('Personal', 'Business', 'Mortgage', 'Auto', 'Education')),
    status VARCHAR(50) NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive')),
    interest_rate_min NUMERIC(5, 2) NOT NULL CHECK (interest_rate_min >= 0),
    interest_rate_max NUMERIC(5, 2) NOT NULL CHECK (interest_rate_max >= interest_rate_min),
    processing_fee_percent NUMERIC(5, 2) DEFAULT 0.00 CHECK (processing_fee_percent >= 0),
    insurance_percent NUMERIC(5, 2) DEFAULT 0.00 CHECK (insurance_percent >= 0),
    tenor_min_months INT NOT NULL CHECK (tenor_min_months > 0),
    tenor_max_months INT NOT NULL CHECK (tenor_max_months >= tenor_min_months),
    min_loan_amount NUMERIC(15, 2) NOT NULL CHECK (min_loan_amount > 0),
    max_loan_amount NUMERIC(15, 2) NOT NULL CHECK (max_loan_amount >= min_loan_amount),
    allow_early_repayment BOOLEAN DEFAULT TRUE NOT NULL,
    allow_partial_repayment BOOLEAN DEFAULT TRUE NOT NULL,
    grace_period_months INT DEFAULT 0 NOT NULL CHECK (grace_period_months >= 0),
    requires_collateral BOOLEAN DEFAULT FALSE NOT NULL,
    requires_guarantors BOOLEAN DEFAULT FALSE NOT NULL,
    min_guarantors INT DEFAULT 0 NOT NULL,
    min_credit_score INT DEFAULT 0 NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_loan_products_modtime ON loan_products;
CREATE TRIGGER update_loan_products_modtime
    BEFORE UPDATE ON loan_products
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE TABLE IF NOT EXISTS loans (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    borrower_id UUID REFERENCES borrowers(id) ON DELETE RESTRICT NOT NULL,
    product_id UUID REFERENCES loan_products(id) ON DELETE RESTRICT NOT NULL,
    amount NUMERIC(15, 2) NOT NULL CHECK (amount > 0),
    tenor INT NOT NULL CHECK (tenor > 0),
    interest_rate NUMERIC(5, 2) NOT NULL CHECK (interest_rate >= 0),
    status VARCHAR(50) NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'Approved', 'Rejected', 'Disbursed', 'Closed')),
    purpose TEXT,
    application_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    approval_date TIMESTAMP WITH TIME ZONE,
    disbursement_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_loans_modtime ON loans;
CREATE TRIGGER update_loans_modtime
    BEFORE UPDATE ON loans
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- Installment Schedules
CREATE TABLE IF NOT EXISTS repayment_schedules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    loan_id UUID REFERENCES loans(id) ON DELETE CASCADE NOT NULL,
    installment_no INT NOT NULL CHECK (installment_no > 0),
    due_date DATE NOT NULL,
    principal NUMERIC(15, 2) NOT NULL CHECK (principal >= 0),
    interest NUMERIC(15, 2) NOT NULL CHECK (interest >= 0),
    total_payment NUMERIC(15, 2) NOT NULL CHECK (total_payment >= 0),
    balance NUMERIC(15, 2) NOT NULL CHECK (balance >= 0),
    status VARCHAR(50) NOT NULL DEFAULT 'Upcoming' CHECK (status IN ('Paid', 'Due', 'Overdue', 'Upcoming')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    UNIQUE (loan_id, installment_no)
);

DROP TRIGGER IF EXISTS update_repayment_schedules_modtime ON repayment_schedules;
CREATE TRIGGER update_repayment_schedules_modtime
    BEFORE UPDATE ON repayment_schedules
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- Recorded payments
CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    loan_id UUID REFERENCES loans(id) ON DELETE RESTRICT NOT NULL,
    payment_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    amount NUMERIC(15, 2) NOT NULL CHECK (amount > 0),
    payment_method VARCHAR(50) NOT NULL CHECK (payment_method IN ('Cash', 'Bank Transfer', 'Mobile Money', 'Cheque')),
    reference_no VARCHAR(100) UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- =========================================================================
-- 5. COLLATERAL REGISTRY
-- =========================================================================

CREATE TABLE IF NOT EXISTS collateral (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    loan_id UUID REFERENCES loans(id) ON DELETE CASCADE NOT NULL,
    borrower_id UUID REFERENCES borrowers(id) ON DELETE RESTRICT NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('Land', 'Building', 'Vehicle', 'Equipment', 'Jewelry', 'Securities', 'Other')),
    description TEXT NOT NULL,
    location VARCHAR(255),
    appraised_value NUMERIC(15, 2) NOT NULL CHECK (appraised_value >= 0),
    registration_number VARCHAR(100),
    registration_date DATE NOT NULL,
    expiry_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'Registered' CHECK (status IN ('Registered', 'Appraised', 'Active', 'Released', 'Seized')),
    insured BOOLEAN DEFAULT FALSE NOT NULL,
    insurance_policy VARCHAR(100),
    insurance_value NUMERIC(15, 2) CHECK (insurance_value >= 0),
    lien BOOLEAN DEFAULT FALSE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_collateral_modtime ON collateral;
CREATE TRIGGER update_collateral_modtime
    BEFORE UPDATE ON collateral
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE TABLE IF NOT EXISTS collateral_appraisals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    collateral_id UUID REFERENCES collateral(id) ON DELETE CASCADE NOT NULL,
    appraiser_name VARCHAR(150) NOT NULL,
    appraisal_date DATE NOT NULL,
    value NUMERIC(15, 2) NOT NULL CHECK (value >= 0),
    condition VARCHAR(50) NOT NULL CHECK (condition IN ('Excellent', 'Good', 'Fair', 'Poor')),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- =========================================================================
-- 6. SAVINGS ENGINE
-- =========================================================================

CREATE TABLE IF NOT EXISTS savings_accounts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    account_number VARCHAR(50) UNIQUE NOT NULL,
    borrower_id UUID REFERENCES borrowers(id) ON DELETE RESTRICT NOT NULL,
    account_type VARCHAR(50) NOT NULL DEFAULT 'Ordinary' CHECK (account_type IN ('Ordinary', 'Fixed Deposit', 'Goal-Based')),
    balance NUMERIC(15, 2) NOT NULL DEFAULT 0.00,
    interest_rate NUMERIC(5, 2) NOT NULL DEFAULT 0.00 CHECK (interest_rate >= 0),
    status VARCHAR(50) NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Dormant', 'Closed')),
    opened_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_transaction_date TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_savings_accounts_modtime ON savings_accounts;
CREATE TRIGGER update_savings_accounts_modtime
    BEFORE UPDATE ON savings_accounts
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE TABLE IF NOT EXISTS savings_transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    account_id UUID REFERENCES savings_accounts(id) ON DELETE CASCADE NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('Deposit', 'Withdrawal', 'Interest', 'Fee')),
    amount NUMERIC(15, 2) NOT NULL CHECK (amount > 0),
    balance_after NUMERIC(15, 2) NOT NULL,
    transaction_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    reference VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- =========================================================================
-- 7. OPERATIONS, WORKFLOWS & EXPENSES
-- =========================================================================

-- Custom Fields for flexibility
DROP TABLE IF EXISTS custom_field_values CASCADE;
DROP TABLE IF EXISTS custom_fields CASCADE;

CREATE TABLE custom_fields (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) UNIQUE NOT NULL,
    label VARCHAR(150) NOT NULL,
    field_type VARCHAR(50) NOT NULL CHECK (field_type IN ('text', 'number', 'date', 'boolean')),
    entity_type VARCHAR(50) NOT NULL CHECK (entity_type IN ('borrower', 'loan', 'savings')),
    required BOOLEAN DEFAULT FALSE NOT NULL,
    options VARCHAR(100)[],
    default_value VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_custom_fields_modtime ON custom_fields;
CREATE TRIGGER update_custom_fields_modtime
    BEFORE UPDATE ON custom_fields
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE TABLE custom_field_values (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    field_id UUID REFERENCES custom_fields(id) ON DELETE CASCADE NOT NULL,
    entity_id UUID NOT NULL, -- borrower_id, loan_id, or savings_account_id
    value TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    UNIQUE (field_id, entity_id)
);

DROP TRIGGER IF EXISTS update_custom_field_values_modtime ON custom_field_values;
CREATE TRIGGER update_custom_field_values_modtime
    BEFORE UPDATE ON custom_field_values
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- Workflows Engine
CREATE TABLE IF NOT EXISTS workflow_definitions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    entity_type VARCHAR(50) NOT NULL CHECK (entity_type IN ('Loan', 'Borrower', 'Collateral', 'Savings')),
    status VARCHAR(50) NOT NULL DEFAULT 'Draft' CHECK (status IN ('Draft', 'Active', 'Paused', 'Completed', 'Cancelled')),
    steps JSONB NOT NULL, -- Array of steps config
    version INT NOT NULL DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_workflow_definitions_modtime ON workflow_definitions;
CREATE TRIGGER update_workflow_definitions_modtime
    BEFORE UPDATE ON workflow_definitions
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE TABLE IF NOT EXISTS workflow_instances (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_id UUID REFERENCES workflow_definitions(id) ON DELETE RESTRICT NOT NULL,
    entity_type VARCHAR(50) NOT NULL CHECK (entity_type IN ('Loan', 'Borrower', 'Collateral', 'Savings')),
    entity_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'Active' CHECK (status IN ('Draft', 'Active', 'Paused', 'Completed', 'Cancelled')),
    current_step INT NOT NULL DEFAULT 1,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_workflow_instances_modtime ON workflow_instances;
CREATE TRIGGER update_workflow_instances_modtime
    BEFORE UPDATE ON workflow_instances
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE TABLE IF NOT EXISTS workflow_tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    instance_id UUID REFERENCES workflow_instances(id) ON DELETE CASCADE NOT NULL,
    step_number INT NOT NULL,
    step_name VARCHAR(100) NOT NULL,
    assigned_to VARCHAR(100) NOT NULL, -- Role or user email
    priority VARCHAR(50) NOT NULL DEFAULT 'Medium' CHECK (priority IN ('Low', 'Medium', 'High', 'Urgent')),
    status VARCHAR(50) NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'InProgress', 'Completed', 'Rejected', 'Escalated')),
    due_date TIMESTAMP WITH TIME ZONE NOT NULL,
    comments TEXT,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_workflow_tasks_modtime ON workflow_tasks;
CREATE TRIGGER update_workflow_tasks_modtime
    BEFORE UPDATE ON workflow_tasks
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- Expense Tracking
CREATE TABLE IF NOT EXISTS expense_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    budget_limit NUMERIC(15, 2),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_expense_categories_modtime ON expense_categories;
CREATE TRIGGER update_expense_categories_modtime
    BEFORE UPDATE ON expense_categories
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE TABLE IF NOT EXISTS expenses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    category_id UUID REFERENCES expense_categories(id) ON DELETE RESTRICT NOT NULL,
    description TEXT NOT NULL,
    amount NUMERIC(15, 2) NOT NULL CHECK (amount > 0),
    date DATE NOT NULL,
    vendor VARCHAR(150),
    reference VARCHAR(100),
    status VARCHAR(50) NOT NULL DEFAULT 'Draft' CHECK (status IN ('Draft', 'Submitted', 'Approved', 'Rejected', 'Paid')),
    attachments TEXT[], -- Array of attachment URLs
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_expenses_modtime ON expenses;
CREATE TRIGGER update_expenses_modtime
    BEFORE UPDATE ON expenses
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- =========================================================================
-- 8. FUNDING & INVESTORS
-- =========================================================================

CREATE TABLE IF NOT EXISTS investors (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(150) NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('Individual', 'Institution', 'Bank', 'Government', 'NGO')),
    contact_person VARCHAR(100),
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(50) NOT NULL,
    country VARCHAR(100) NOT NULL,
    website VARCHAR(255),
    registration_number VARCHAR(100),
    status VARCHAR(50) NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_investors_modtime ON investors;
CREATE TRIGGER update_investors_modtime
    BEFORE UPDATE ON investors
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE TABLE IF NOT EXISTS investments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    investor_id UUID REFERENCES investors(id) ON DELETE RESTRICT NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('Equity', 'Debt', 'Grant', 'Guarantee')),
    amount NUMERIC(15, 2) NOT NULL CHECK (amount > 0),
    currency VARCHAR(10) NOT NULL DEFAULT 'UGX',
    term INT NOT NULL CHECK (term > 0), -- term in months
    interest_rate NUMERIC(5, 2),
    start_date DATE NOT NULL,
    maturity_date DATE,
    status VARCHAR(50) NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive', 'Exited', 'Matured')),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_investments_modtime ON investments;
CREATE TRIGGER update_investments_modtime
    BEFORE UPDATE ON investments
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- =========================================================================
-- 9. COMMUNICATIONS & AUDIT
-- =========================================================================

DROP TABLE IF EXISTS notifications CASCADE;

CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    type VARCHAR(100) NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    priority VARCHAR(50) NOT NULL DEFAULT 'Medium',
    channels VARCHAR(100)[],
    status VARCHAR(50) NOT NULL DEFAULT 'New',
    sent_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- =========================================================================
-- 10. INDEXES FOR PERFORMANCE OPTIMIZATION
-- =========================================================================

-- Users search index
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

-- Borrowers search indexes
CREATE INDEX IF NOT EXISTS idx_borrowers_names ON borrowers(first_name, last_name);
CREATE INDEX IF NOT EXISTS idx_borrowers_email ON borrowers(email);
CREATE INDEX IF NOT EXISTS idx_borrowers_phone ON borrowers(phone);
CREATE INDEX IF NOT EXISTS idx_borrowers_id_number ON borrowers(id_number);
CREATE INDEX IF NOT EXISTS idx_borrowers_kyc ON borrowers(kyc_status);

-- Loans query indexes
CREATE INDEX IF NOT EXISTS idx_loans_borrower ON loans(borrower_id);
CREATE INDEX IF NOT EXISTS idx_loans_product ON loans(product_id);
CREATE INDEX IF NOT EXISTS idx_loans_status ON loans(status);

-- Repayment schedule lookup index
CREATE INDEX IF NOT EXISTS idx_repayment_schedules_loan ON repayment_schedules(loan_id);
CREATE INDEX IF NOT EXISTS idx_repayment_schedules_due ON repayment_schedules(due_date);
CREATE INDEX IF NOT EXISTS idx_repayment_schedules_status ON repayment_schedules(status);

-- Savings accounts lookup index
CREATE INDEX IF NOT EXISTS idx_savings_accounts_num ON savings_accounts(account_number);
CREATE INDEX IF NOT EXISTS idx_savings_accounts_borrower ON savings_accounts(borrower_id);

-- Savings transactions index
CREATE INDEX IF NOT EXISTS idx_savings_transactions_acc ON savings_transactions(account_id);

-- Custom fields lookup index
CREATE INDEX IF NOT EXISTS idx_custom_field_values_entity ON custom_field_values(entity_id);

-- Workflows indexes
CREATE INDEX IF NOT EXISTS idx_workflow_instances_entity ON workflow_instances(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_workflow_tasks_instance ON workflow_tasks(instance_id);
CREATE INDEX IF NOT EXISTS idx_workflow_tasks_assignee ON workflow_tasks(assigned_to);

-- Expenses indexes
CREATE INDEX IF NOT EXISTS idx_expenses_category ON expenses(category_id);
CREATE INDEX IF NOT EXISTS idx_expenses_status ON expenses(status);

-- Investments indexes
CREATE INDEX IF NOT EXISTS idx_investments_investor ON investments(investor_id);


-- =========================================================================
-- 11. INITIAL SEED DATA
-- =========================================================================

-- Default Admin User (Password is 'password123' hashed with bcrypt)
INSERT INTO users (id, first_name, last_name, email, password_hash, role, status, department)
VALUES (
    'a3b8d4e9-0123-4567-89ab-cdef01234567',
    'System',
    'Admin',
    'admin@lmspro.com',
    '$2b$12$K38ZfB2d2e1B6X1B8mDqIe2621C4G7xV9aTjG8nF3nE5sY9tNfK6G',
    'Admin',
    'Active',
    'Information Technology'
) ON CONFLICT (email) DO NOTHING;

-- Initial Loan Products
INSERT INTO loan_products (name, description, type, status, interest_rate_min, interest_rate_max, processing_fee_percent, insurance_percent, tenor_min_months, tenor_max_months, min_loan_amount, max_loan_amount, allow_early_repayment, allow_partial_repayment, requires_collateral, requires_guarantors, min_guarantors, min_credit_score)
SELECT 'Emergency Personal Loan', 'Short-term personal loan for emergency expenses.', 'Personal', 'Active', 10.00, 15.00, 2.00, 1.00, 1, 12, 100000.00, 2000000.00, true, true, false, true, 1, 550
WHERE NOT EXISTS (SELECT 1 FROM loan_products WHERE name = 'Emergency Personal Loan');

INSERT INTO loan_products (name, description, type, status, interest_rate_min, interest_rate_max, processing_fee_percent, insurance_percent, tenor_min_months, tenor_max_months, min_loan_amount, max_loan_amount, allow_early_repayment, allow_partial_repayment, requires_collateral, requires_guarantors, min_guarantors, min_credit_score)
SELECT 'SME Business Growth Fund', 'Medium-term business expansion funding.', 'Business', 'Active', 12.00, 18.00, 3.00, 1.50, 6, 36, 1000000.00, 50000000.00, true, true, true, true, 2, 600
WHERE NOT EXISTS (SELECT 1 FROM loan_products WHERE name = 'SME Business Growth Fund');

INSERT INTO loan_products (name, description, type, status, interest_rate_min, interest_rate_max, processing_fee_percent, insurance_percent, tenor_min_months, tenor_max_months, min_loan_amount, max_loan_amount, allow_early_repayment, allow_partial_repayment, requires_collateral, requires_guarantors, min_guarantors, min_credit_score)
SELECT 'First-Time Home Buyers', 'Long-term residential property mortgage.', 'Mortgage', 'Active', 8.00, 12.00, 1.50, 0.50, 60, 240, 10000000.00, 200000000.00, true, false, true, false, 0, 650
WHERE NOT EXISTS (SELECT 1 FROM loan_products WHERE name = 'First-Time Home Buyers');

-- Default Expense Categories
INSERT INTO expense_categories (code, name, description, budget_limit)
VALUES 
('OPE-RENT', 'Office Rent', 'Monthly workspace lease payments', 5000000.00),
('OPE-UTIL', 'Utilities', 'Electricity, water, internet connectivity charges', 1500000.00),
('OPE-STAFF', 'Staff Lunches & Travel', 'Meals, team outings, and official mileage allowance', 3000000.00),
('OPE-MARK', 'Marketing & Sales', 'Social media ads, flyers, print media campaigns', 10000000.00),
('OPE-MISC', 'Miscellaneous', 'General unclassified office operational items', 1000000.00)
ON CONFLICT (code) DO NOTHING;

-- Initial Workflow Definition for Loan Approval
INSERT INTO workflow_definitions (name, description, entity_type, status, steps, version)
SELECT 
    'Standard Loan Approval Process',
    'Multi-stage evaluation flow for new credit applications.',
    'Loan',
    'Active',
    '[
        {
            "stepNumber": 1,
            "name": "Document Verification",
            "description": "Verify borrower KYC documents and collateral proofs.",
            "assignedRole": "Loan Officer",
            "action": "Verify Docs",
            "timeLimit": 24,
            "mandatory": true,
            "nextSteps": [{"condition": "Approved", "stepNumber": 2}]
        },
        {
            "stepNumber": 2,
            "name": "Risk & Credit Scoring",
            "description": "Calculate financial ratios and verify CRB score.",
            "assignedRole": "Manager",
            "action": "Appraise Risk",
            "timeLimit": 48,
            "mandatory": true,
            "nextSteps": [{"condition": "Approved", "stepNumber": 3}]
        },
        {
            "stepNumber": 3,
            "name": "Final Sign-off",
            "description": "Approve final release terms and disburse funds.",
            "assignedRole": "Admin",
            "action": "Authorize Disbursement",
            "timeLimit": 12,
            "mandatory": true,
            "nextSteps": []
        }
    ]'::jsonb,
    1
WHERE NOT EXISTS (SELECT 1 FROM workflow_definitions WHERE name = 'Standard Loan Approval Process');

-- =========================================================================
-- 12. ACCOUNTING & GENERAL LEDGER
-- =========================================================================

CREATE TABLE IF NOT EXISTS general_ledger_entries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    account_code VARCHAR(50) NOT NULL,
    account_name VARCHAR(150) NOT NULL,
    debit NUMERIC(15, 2) NOT NULL DEFAULT 0.00 CHECK (debit >= 0),
    credit NUMERIC(15, 2) NOT NULL DEFAULT 0.00 CHECK (credit >= 0),
    reference VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    module VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS other_incomes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    category VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    related_loan_id UUID REFERENCES loans(id) ON DELETE SET NULL,
    related_borrower_id UUID REFERENCES borrowers(id) ON DELETE SET NULL,
    amount NUMERIC(15, 2) NOT NULL CHECK (amount > 0),
    date DATE NOT NULL DEFAULT CURRENT_DATE,
    gl_account VARCHAR(100) NOT NULL,
    reference VARCHAR(100) NOT NULL UNIQUE,
    status VARCHAR(50) NOT NULL DEFAULT 'Recorded' CHECK (status IN ('Recorded', 'Verified', 'Posted')),
    recorded_by VARCHAR(150) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_other_incomes_modtime ON other_incomes;
CREATE TRIGGER update_other_incomes_modtime
    BEFORE UPDATE ON other_incomes
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_gl_entries_code ON general_ledger_entries(account_code);
CREATE INDEX IF NOT EXISTS idx_gl_entries_reference ON general_ledger_entries(reference);
CREATE INDEX IF NOT EXISTS idx_other_incomes_status ON other_incomes(status);

-- Seed initial ledger entries
INSERT INTO general_ledger_entries (date, account_code, account_name, debit, credit, reference, description, module)
SELECT '2026-07-01', '1001', 'Cash at Bank', 15000000.00, 0.00, 'OPEN-2026', 'Opening Cash Balance', 'General Ledger'
WHERE NOT EXISTS (SELECT 1 FROM general_ledger_entries WHERE reference = 'OPEN-2026' AND account_code = '1001');

INSERT INTO general_ledger_entries (date, account_code, account_name, debit, credit, reference, description, module)
SELECT '2026-07-01', '3001', 'Retained Earnings', 0.00, 15000000.00, 'OPEN-2026', 'Opening Capital Balance', 'General Ledger'
WHERE NOT EXISTS (SELECT 1 FROM general_ledger_entries WHERE reference = 'OPEN-2026' AND account_code = '3001');

-- Seed initial other income records
INSERT INTO other_incomes (category, description, amount, gl_account, reference, status, recorded_by)
VALUES 
('Processing Fee', 'Loan processing charge for LN-001', 5000.00, '4001', 'INC-001', 'Posted', 'admin@lmspro.com'),
('Penalty Income', 'Late payment fine for LN-002', 10000.00, '4002', 'INC-002', 'Verified', 'admin@lmspro.com')
ON CONFLICT (reference) DO NOTHING;


-- =========================================================================
-- 13. SYSTEM SETTINGS & AUDIT LOGGING
-- =========================================================================

CREATE TABLE IF NOT EXISTS branches (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    code VARCHAR(50) UNIQUE NOT NULL,
    address TEXT,
    city VARCHAR(100),
    phone VARCHAR(50),
    email VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_branches_modtime ON branches;
CREATE TRIGGER update_branches_modtime
    BEFORE UPDATE ON branches
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE TABLE IF NOT EXISTS staff (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    branch_id UUID REFERENCES branches(id) ON DELETE RESTRICT NOT NULL,
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    phone VARCHAR(50) NOT NULL,
    role VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Inactive')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_staff_modtime ON staff;
CREATE TRIGGER update_staff_modtime
    BEFORE UPDATE ON staff
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(255) NOT NULL,
    entity_type VARCHAR(100),
    entity_id UUID,
    old_values JSONB,
    new_values JSONB,
    ip_address VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_staff_branch ON staff(branch_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);

-- Seed initial branch
INSERT INTO branches (id, name, code, address, city, phone, email, status)
SELECT 'a3b8d4e9-0123-4567-89ab-cdef01234568', 'Main Branch', 'HQ-001', '123 Headquarters Boulevard', 'Kampala', '+256701234567', 'hq@lmspro.com', 'Active'
WHERE NOT EXISTS (SELECT 1 FROM branches WHERE code = 'HQ-001');

-- Seed initial staff
INSERT INTO staff (branch_id, first_name, last_name, email, phone, role, status)
SELECT 'a3b8d4e9-0123-4567-89ab-cdef01234568', 'System', 'Admin', 'admin@lmspro.com', '+256701234567', 'Administrator', 'Active'
WHERE NOT EXISTS (SELECT 1 FROM staff WHERE email = 'admin@lmspro.com');


-- =========================================================================
-- 14. PAYMENT ARRANGEMENTS / LOAN RESTRUCTURING
-- =========================================================================

CREATE TABLE IF NOT EXISTS payment_arrangements (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    loan_id UUID REFERENCES loans(id) ON DELETE CASCADE NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('Refinance', 'Reschedule', 'Write-off', 'Settlement')),
    proposed_amount DECIMAL(15, 2) NOT NULL,
    revised_tenor INTEGER NOT NULL,
    revised_interest_rate DECIMAL(5, 2) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'Proposed' CHECK (status IN ('Proposed', 'Approved', 'Rejected', 'Active', 'Closed')),
    reason TEXT,
    approved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    approved_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_payment_arrangements_modtime ON payment_arrangements;
CREATE TRIGGER update_payment_arrangements_modtime
    BEFORE UPDATE ON payment_arrangements
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE INDEX IF NOT EXISTS idx_payment_arrangements_loan ON payment_arrangements(loan_id);


-- =========================================================================
-- 15. COMMUNICATIONS
-- =========================================================================

CREATE TABLE IF NOT EXISTS communication_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(150) UNIQUE NOT NULL,
    subject VARCHAR(255),
    body TEXT NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('Email', 'SMS', 'WhatsApp')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_communication_templates_modtime ON communication_templates;
CREATE TRIGGER update_communication_templates_modtime
    BEFORE UPDATE ON communication_templates
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();

CREATE TABLE IF NOT EXISTS communications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    borrower_id UUID REFERENCES borrowers(id) ON DELETE CASCADE NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('Email', 'SMS', 'WhatsApp')),
    subject VARCHAR(255),
    body TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'Sent', 'Failed')),
    sent_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_communications_borrower ON communications(borrower_id);

-- Seed initial communication templates
INSERT INTO communication_templates (id, name, subject, body, type)
VALUES 
('b1a8d4e9-0123-4567-89ab-cdef01234560', 'Loan Approval Notice', 'Your loan has been approved!', 'Dear client, your loan application has been approved. Funds will be disbursed shortly.', 'Email'),
('b1a8d4e9-0123-4567-89ab-cdef01234561', 'Repayment Reminder SMS', NULL, 'Reminder: Your loan installment is due in 3 days. Please ensure sufficient funds.', 'SMS')
ON CONFLICT (name) DO NOTHING;


-- =========================================================================
-- 16. DOCUMENTS Registry
-- =========================================================================

CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    file_type VARCHAR(100) NOT NULL,
    file_size INT NOT NULL,
    file_url VARCHAR(255) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    uploaded_by VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'Pending' CHECK (status IN ('Pending', 'Approved', 'Rejected')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

DROP TRIGGER IF EXISTS update_documents_modtime ON documents;
CREATE TRIGGER update_documents_modtime
    BEFORE UPDATE ON documents
    FOR EACH ROW
    EXECUTE FUNCTION update_modified_column();






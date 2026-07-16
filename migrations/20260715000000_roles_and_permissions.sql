-- Migration to create roles and permissions tables
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT NOT NULL,
    permissions VARCHAR(100)[] NOT NULL DEFAULT '{}',
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Seed default system roles
INSERT INTO roles (id, name, description, permissions, active) VALUES
('1a1a1a1a-1a1a-1a1a-1a1a-1a1a1a1a1a1a', 'Admin', 'Full system administrator access with all permissions.', ARRAY['borrower.view', 'borrower.create', 'borrower.edit', 'borrower.delete', 'borrower.kyc', 'loan.view', 'loan.create', 'loan.approve', 'loan.disburse', 'repayment.view', 'repayment.create', 'repayment.bulk', 'savings.view', 'savings.deposit', 'savings.withdraw', 'savings.transfer', 'settings.manage', 'roles.manage', 'staff.manage', 'audit.view', 'report.view', 'report.export', 'collateral.view', 'document.view'], true),
('2b2b2b2b-2b2b-2b2b-2b2b-2b2b2b2b2b2b', 'Manager', 'Operations manager. Can approve loans, manage borrowers, and view reports.', ARRAY['borrower.view', 'borrower.create', 'borrower.edit', 'loan.view', 'loan.approve', 'repayment.view', 'repayment.create', 'savings.view', 'savings.deposit', 'report.view', 'collateral.view', 'document.view'], true),
('3c3c3c3c-3c3c-3c3c-3c3c-3c3c3c3c3c3c', 'Loan Officer', 'Can register borrowers, initiate loans, and upload documents.', ARRAY['borrower.view', 'borrower.create', 'borrower.kyc', 'loan.view', 'loan.create', 'repayment.view', 'repayment.create', 'collateral.view', 'document.view'], true),
('4d4d4d4d-4d4d-4d4d-4d4d-4d4d4d4d4d4d', 'Auditor', 'Read-only access for compliance auditing and log inspection.', ARRAY['borrower.view', 'loan.view', 'repayment.view', 'savings.view', 'report.view', 'audit.view', 'collateral.view'], true)
ON CONFLICT (name) DO NOTHING;

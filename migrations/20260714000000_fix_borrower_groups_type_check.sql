-- Drop strict check constraint on borrower group type to allow any frontend group type strings
ALTER TABLE borrower_groups DROP CONSTRAINT IF EXISTS borrower_groups_type_check;

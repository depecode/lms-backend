use actix_web::{web, HttpResponse};
mod health;
use crate::handlers::{
    auth_handler, borrower_handler, loan_handler, loan_product_handler,
    repayment_handler, savings_handler, accounting_handler, report_handler,
    setting_handler, document_handler, workflow_handler, investor_handler,
    notification_handler, user_handler
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::models::response::{ApiResponse, ResponseMeta, PaginatedResponse, PaginatedMeta, ApiError, ErrorDetail};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth_handler::login,
        auth_handler::refresh,
        auth_handler::profile,
        auth_handler::logout,
        borrower_handler::list_borrowers,
        borrower_handler::create_borrower,
        borrower_handler::get_borrower,
        borrower_handler::update_borrower,
        borrower_handler::list_groups,
        borrower_handler::list_guarantors,
        loan_handler::list_loans,
        loan_handler::submit_loan,
        loan_handler::get_loan,
        loan_handler::approve_loan,
        loan_handler::disburse_loan,
        loan_handler::get_loan_scoring,
        loan_product_handler::list_products,
        loan_product_handler::create_product,
        loan_product_handler::get_product,
        repayment_handler::list_repayments,
        repayment_handler::record_payment,
        repayment_handler::list_arrears,
        savings_handler::list_savings_accounts,
        savings_handler::bulk_upload_deposits,
        savings_handler::get_account_history,
        accounting_handler::get_ledger,
        accounting_handler::get_statements,
        accounting_handler::record_other_income,
        report_handler::get_portfolio_summary,
        report_handler::get_loan_stats,
        report_handler::export_report,
        setting_handler::list_branches,
        setting_handler::create_branch,
        setting_handler::list_staff,
        setting_handler::create_staff,
        setting_handler::get_audit_logs,
        document_handler::upload_doc,
        workflow_handler::list_tasks,
        investor_handler::list_investors,
        notification_handler::list_notifications,
        user_handler::create_user,
        user_handler::get_users,
        user_handler::get_user,
        user_handler::update_user,
        user_handler::delete_user,
        health::health_check,
    ),
    components(
        schemas(
            auth_handler::LoginRequest,
            ApiResponse<serde_json::Value>,
            ResponseMeta,
            PaginatedResponse<serde_json::Value>,
            PaginatedMeta,
            ApiError,
            ErrorDetail,
        )
    ),
    tags(
        (name = "LMS API", description = "Loan Management System API")
    )
)]
struct ApiDoc;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(health::health_check)
            .service(
                web::scope("/auth")
                    .service(auth_handler::login)
                    .service(auth_handler::refresh)
                    .service(auth_handler::profile)
                    .service(auth_handler::logout)
            )
            .service(
                web::scope("/borrowers")
                    .service(borrower_handler::list_borrowers)
                    .service(borrower_handler::create_borrower)
                    .service(borrower_handler::get_borrower)
                    .service(borrower_handler::update_borrower)
                    .service(borrower_handler::list_groups)
                    .service(borrower_handler::list_guarantors)
            )
            .service(
                web::scope("/loans")
                    .service(loan_handler::list_loans)
                    .service(loan_handler::submit_loan)
                    .service(loan_handler::get_loan)
                    .service(loan_handler::approve_loan)
                    .service(loan_handler::disburse_loan)
                    .service(loan_handler::get_loan_scoring)
            )
            .service(
                web::scope("/loan-products")
                    .service(loan_product_handler::list_products)
                    .service(loan_product_handler::create_product)
                    .service(loan_product_handler::get_product)
            )
            .service(
                web::scope("/repayments")
                    .service(repayment_handler::list_repayments)
                    .service(repayment_handler::record_payment)
                    .service(repayment_handler::list_arrears)
            )
            .service(
                web::scope("/savings")
                    .service(savings_handler::list_savings_accounts)
                    .service(savings_handler::bulk_upload_deposits)
                    .service(savings_handler::get_account_history)
            )
            .service(
                web::scope("/accounting")
                    .service(accounting_handler::get_ledger)
                    .service(accounting_handler::get_statements)
                    .service(accounting_handler::record_other_income)
            )
            .service(
                web::scope("/reports")
                    .service(report_handler::get_portfolio_summary)
                    .service(report_handler::get_loan_stats)
                    .service(report_handler::export_report)
            )
            .service(
                web::scope("/settings")
                    .service(setting_handler::list_branches)
                    .service(setting_handler::create_branch)
                    .service(setting_handler::list_staff)
                    .service(setting_handler::create_staff)
                    .service(setting_handler::get_audit_logs)
            )
            .service(
                web::scope("/docs")
                    .service(document_handler::upload_doc)
            )
            .service(
                web::scope("/workflows")
                    .service(workflow_handler::list_tasks)
            )
            .service(
                web::scope("/investors")
                    .service(investor_handler::list_investors)
            )
            .service(
                web::scope("/notifications")
                    .service(notification_handler::list_notifications)
            )
            .service(
                web::scope("/users")
                    .service(user_handler::create_user)
                    .service(user_handler::get_users)
                    .service(user_handler::get_user)
                    .service(user_handler::update_user)
                    .service(user_handler::delete_user)
            )
            .service(
                web::scope("/health")
                    .service(health::health_check)
            )
    )
    .service(
        SwaggerUi::new("/swagger-ui/{_:.*}")
            .url("/api-docs/openapi.json", ApiDoc::openapi())
    )
    .route("/swagger-ui", web::get().to(|| async {
        HttpResponse::Found()
            .append_header(("Location", "/swagger-ui/"))
            .finish()
    }));
}

use actix_web::web;

mod health;
use crate::handlers::{user_handler, auth_handler};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(health::health_check);
    cfg.service(user_handler::create_user);
    cfg.service(user_handler::get_users);
    cfg.service(user_handler::get_user);
    cfg.service(user_handler::update_user);
    cfg.service(user_handler::delete_user);
}

use actix_web::{get, put, web, HttpResponse, Responder};

#[get("/config")]
pub async fn get_config() -> impl Responder {
    let cfg = config::get_config().await;
    HttpResponse::Ok().json(cfg)
}

#[put("/config/{name}")]
pub async fn put_node_name(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    let mut cfg = config::get_config().await;
    cfg.set_node_name(name).await;
    HttpResponse::Ok().json(cfg)
}

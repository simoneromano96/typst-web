use actix_web::{post, web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use utoipa::{OpenApi, ToSchema};
use utoipa_scalar::{Scalar, Servable};

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CompileRequest {
    template: String,
    variables: Option<HashMap<String, String>>,
    jobs: Option<u32>,
}

#[utoipa::path(
    post,
    path = "/api/typst/compile",
    request_body = CompileRequest,
    responses(
        (status = 200, description = "Compiled PDF", content_type = "application/pdf", body = [u8]),
        (status = 500, description = "Error", body = String)
    )
)]
#[post("/api/typst/compile")]
async fn compile_pdf(body: web::Json<CompileRequest>) -> Result<HttpResponse, actix_web::Error> {
    let mut cmd = Command::new("typst");

    cmd.args(["compile", "-", "-"]); //.arg("-").arg("-");
    if let Some(j) = &body.jobs {
        cmd.arg("--jobs").arg(j.to_string());
    }
    if let Some(vars) = &body.variables {
        for (key, value) in vars {
            cmd.arg("--input").arg(format!("{}={}", key, value));
        }
    }
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(body.template.as_bytes())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    if output.status.success() {
        Ok(HttpResponse::Ok()
            .content_type("application/pdf")
            .body(output.stdout))
    } else {
        Ok(HttpResponse::InternalServerError()
            .body(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

#[derive(OpenApi)]
#[openapi(paths(compile_pdf), components(schemas(CompileRequest)))]
struct ApiDoc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(compile_pdf)
            .service(Scalar::with_url("/scalar", ApiDoc::openapi()))
            .route(
                "/api-docs/openapi.json",
                web::get().to(|| async { actix_web::HttpResponse::Ok().json(ApiDoc::openapi()) }),
            )
    })
    .bind(("127.0.0.1", 3030))?
    .run()
    .await
}

use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, env, process::Stdio};
use tokio::{io::AsyncWriteExt, net::TcpListener, process::Command};
use utoipa::{OpenApi, ToSchema};

#[derive(Deserialize, Serialize, ToSchema)]
struct CompileRequest {
    /// Template that will be rendered
    #[schema(example = "Hello, #sys.input.name!")]
    template: String,
    /// Variables that will be passed to the template, accessible in the template via #sys.input.variable_name
    ///
    /// Important: The variable name cannot contain underscores, use lowerCamelCase as a convention
    #[schema(example = "{\"name\": \"John Doe\"}")]
    variables: Option<HashMap<String, String>>,
    /// Number of threads to use for rendering, defaults to the number of logical CPUs
    #[schema(example = 4, minimum = 1, maximum = 256)]
    jobs: Option<u32>,
}

#[derive(Serialize, ToSchema)]
#[schema(
    format = "Binary",
    content_encoding = "binary",
    content_media_type = "application/pdf",
    value_type = String
)]
struct CompiledPdf(Vec<u8>);

impl IntoResponse for CompiledPdf {
    fn into_response(self) -> Response {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/pdf")],
            self.0,
        )
            .into_response()
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
enum CompileError {
    /// Error when reading or writing files, spawning processes, etc.
    #[schema(example = "Todo already exists")]
    IoError(String),
    /// Error when the template is invalid, for example, a syntax error, unexpected token, missing variable, etc.
    #[schema(example = "Invalid template: unexpected token '!'")]
    InvalidTemplate(String),
}

impl IntoResponse for CompileError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            CompileError::IoError(error) => (StatusCode::INTERNAL_SERVER_ERROR, error),
            CompileError::InvalidTemplate(error) => (StatusCode::BAD_REQUEST, error),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[utoipa::path(
    post,
    path = "/api/typst/compile",
    request_body = CompileRequest,
    responses(
        (status = 200, description = "Compiled PDF", content_type = "application/pdf", body = CompiledPdf),
        (status = 500, description = "Error", body = CompileError)
    )
)]
async fn compile_pdf(Json(body): Json<CompileRequest>) -> Result<CompiledPdf, CompileError> {
    let mut cmd = Command::new("typst");
    cmd.args(["compile", "-", "-"]);
    if let Some(j) = body.jobs {
        cmd.arg("--jobs").arg(j.to_string());
    }
    if let Some(vars) = body.variables {
        for (key, value) in vars {
            cmd.arg("--input").arg(format!("{}={}", key, value));
        }
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| CompileError::IoError(e.to_string()))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(body.template.as_bytes())
            .await
            .map_err(|e| CompileError::IoError(e.to_string()))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| CompileError::IoError(e.to_string()))?;

    if output.status.success() == false {
        let error_message = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(CompileError::InvalidTemplate(error_message));
    }

    Ok(CompiledPdf(output.stdout))
}

#[derive(OpenApi)]
#[openapi(paths(compile_pdf), components(schemas(CompileRequest)))]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/typst/compile", post(compile_pdf))
        .route(
            "/api/docs/openapi.json",
            get(|| async { Json(ApiDoc::openapi()) }),
        );

    let port = env::var("APP_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!(":::{}", port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

//! API tests: run against a temporary PostgreSQL container.

use std::env;
use std::sync::Arc;

use actix_web::test;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use serde_json::json;
use testcontainers::clients;
use testcontainers::images::postgres;

use actix_clean_architecture::container::Container;
use actix_clean_architecture::data::service_context::dto::ServiceContextDTO;
use actix_clean_architecture::data::service_context::repository::ServiceContextRepository;
use actix_clean_architecture::domain::use_cases::todos::{
    CreateTodoResponse, GetTodoResponse, ListTodosResponse,
};
use actix_clean_architecture::infrastructure::clients::postgres::engine::{db_pool, init_database};
use actix_clean_architecture::infrastructure::core::application::create_app;
use actix_clean_architecture::infrastructure::core::logging_config::init_logging;
use actix_clean_architecture::infrastructure::core::settings::{get_config, POSTGRESQL_DSN_ENV};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[actix_web::test]
async fn test_todos() {
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("RUST_LOG", "debug");

    let docker = clients::Cli::default();
    let postgres_node = docker.run(postgres::Postgres::default());
    let connection_string = &format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        postgres_node.get_host_port_ipv4(5432)
    );
    env::set_var(POSTGRESQL_DSN_ENV, connection_string);

    let config = get_config();
    init_logging(config);
    init_database(config);
    db_pool()
        .get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();

    let container = Arc::new(Container::new());
    let app = test::init_service(create_app(container)).await;

    let request_body = json!({
        "title": "test todo",
        "description": "Test description"
    });

    // Creation test
    let resp = test::TestRequest::post()
        .uri("/todos")
        .set_json(&request_body)
        .send_request(&app)
        .await;
    assert!(resp.status().is_success());
    let todo: CreateTodoResponse = test::read_body_json(resp).await;
    assert_eq!(todo.title, "test todo");
    assert_eq!(todo.description, "Test description");
    assert!(!todo.completed);

    // Get by id test
    let resp = test::TestRequest::get()
        .uri(&format!("/todos/{}", todo.id))
        .send_request(&app)
        .await;
    assert!(resp.status().is_success());
    let retrieved_todo: GetTodoResponse = test::read_body_json(resp).await;
    assert_eq!(todo.id, retrieved_todo.id);
    assert_eq!(todo.title, retrieved_todo.title);

    // Get missing todo test: 400 in the common error body, with the trace id
    let resp = test::TestRequest::get()
        .uri("/todos/99999")
        .send_request(&app)
        .await;
    assert_eq!(resp.status(), 400);
    let trace_id = resp
        .headers()
        .get("x-trace-id")
        .expect("trace id response header")
        .to_str()
        .unwrap()
        .to_string();
    let error: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(error["error"], "BAD_REQUEST");
    assert!(error["details"].is_string());
    assert_eq!(error["trace_id"], json!(trace_id));
    assert_eq!(trace_id.len(), 32);
    assert!(error["timestamp"].is_string());

    // Validation test: a malformed JSON body is a 400 VALIDATION_ERROR
    let resp = test::TestRequest::post()
        .uri("/todos")
        .insert_header(("content-type", "application/json"))
        .set_payload("{bad json")
        .send_request(&app)
        .await;
    assert_eq!(resp.status(), 400);
    let error: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(error["error"], "VALIDATION_ERROR");

    // Trace id test: an incoming `traceparent` header is continued
    let given_trace_id = "4bf92f3577b34da6a3ce929d0e0e4736";
    let resp = test::TestRequest::post()
        .uri("/todos")
        .insert_header((
            "traceparent",
            format!("00-{given_trace_id}-00f067aa0ba902b7-00"),
        ))
        .set_json(&request_body)
        .send_request(&app)
        .await;
    assert!(resp.status().is_success());
    assert_eq!(
        resp.headers()
            .get("x-trace-id")
            .and_then(|value| value.to_str().ok()),
        Some(given_trace_id)
    );

    // Second creation
    let resp = test::TestRequest::post()
        .uri("/todos")
        .set_json(&request_body)
        .send_request(&app)
        .await;
    assert!(resp.status().is_success());

    // List test
    let resp = test::TestRequest::get()
        .uri("/todos")
        .send_request(&app)
        .await;
    assert!(resp.status().is_success());
    let todos: ListTodosResponse = test::read_body_json(resp).await;
    assert_eq!(todos.items.len(), 3);

    // Delete test
    let resp = test::TestRequest::delete()
        .uri(&format!("/todos/{}", todo.id))
        .send_request(&app)
        .await;
    assert!(resp.status().is_success());

    // List after delete test
    let resp = test::TestRequest::get()
        .uri("/todos")
        .send_request(&app)
        .await;
    assert!(resp.status().is_success());
    let todos: ListTodosResponse = test::read_body_json(resp).await;
    assert_eq!(todos.items.len(), 2);

    // Maintenance test: every request is answered 503 while the maintenance window is active
    let service_context_repository = ServiceContextRepository::new(db_pool());
    service_context_repository
        .update(ServiceContextDTO {
            id: 1,
            maintenance: true,
        })
        .await
        .unwrap();
    let resp = test::TestRequest::get()
        .uri("/todos")
        .send_request(&app)
        .await;
    assert_eq!(resp.status(), 503);
    service_context_repository
        .update(ServiceContextDTO {
            id: 1,
            maintenance: false,
        })
        .await
        .unwrap();

    // The maintenance mode is off again
    let resp = test::TestRequest::get()
        .uri("/todos")
        .send_request(&app)
        .await;
    assert!(resp.status().is_success());
}

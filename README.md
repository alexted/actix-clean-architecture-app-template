# Cookiecutter actix simple clean architecture
This is a reusable Rust Cookiecutter template. The project is based on Actix web in combination with Diesel ORM.

Complete list of features the template provides:
* Layered (onion) architecture: `api` / `domain` / `data` / `infrastructure`
* Actix Web
* Use-case driven business logic
* Common error response format with trace ids
* Request logging middleware
* Maintenance window support
* Diesel ORM
* Database migrations
* Local postgres database docker support
* Test containers integration for testing

## Getting started
To start a new project, run the following command:
```bash
cookiecutter https://github.com/microsoft/cookiecutter-rust-actix-clean-architecture
```
This will prompt you for some information about your project. The information
you provide will be used to populate the files in the new project directory.

You can then build the project locally.
```bash
cargo build
```

## Architecture 
The application follows the Onion Architecture pattern. An article is written 
about our experience integrating an onion architecture with actix web in combination with diesel ORM that can 
be found [here](./docs/onion-architecture-article.md).

This architecture is a design pattern that organizes the codebase of a software application into multiple layers, where the innermost layer 
is the domain layer and the outermost layer is the infrastructure layer. Each layer depends only on the layers inside of it and not on the layers outside of it, 
creating a separation of concerns, allowing for a more maintainable and scalable codebase.

The application consists of four layers:

* `api` module: The web layer: the endpoints of the application. The handlers are thin: 
  they parse the request into a use-case request model, execute the use case and return 
  its response model.
* `domain` module: The business layer: the use cases of the application with their 
  request and response models, built on the common `BaseUseCase` contract. The 
  `domain/interfaces` module is a placeholder for contracts that the business logic 
  defines for outer layers.
* `data` module: Repositories that execute the storage statements and return DTOs. 
  `data/base.rs` holds the common repository contract (`BaseRepository`), the `QueryParams` 
  paging contract, the repository error type and the `ResultPaging` page model.
* `infrastructure` module: The outermost layer: `core` contains the application 
  wiring (settings, application factory, error handling, middlewares, logging), 
  `clients` contains the database access (the PostgreSQL connection pool, the Diesel 
  schema and the Diesel models).

The dependencies are composed once in the `Container` (`src/container.rs`) and published 
as the shared state of the application by the app factory, so the endpoints build their 
use cases from them for every request.

Folder structure of the application (`src`):
```
.
├── api
│   ├── mod.rs           # routes registration
│   └── todos.rs         # todos CRUD endpoints
├── domain
│   ├── interfaces       # placeholder: contracts owned by the business logic
│   └── use_cases
│       ├── base.rs      # the BaseUseCase contract
│       └── todos        # one module per todos operation
├── data
│   ├── base.rs          # repository base contract and errors
│   ├── todos            # todos repository + DTO
│   └── service_context  # service context repository + DTO
├── infrastructure
│   ├── core
│   │   ├── settings.rs           # application configuration from the environment
│   │   ├── application.rs        # application factory (takes the container)
│   │   ├── logging_config.rs     # logfmt logging setup
│   │   ├── errors                # error types and the common error body
│   │   └── middlewares           # error handling, log/maintenance/trace-id middlewares
│   └── clients
│       └── postgres              # connection pool, Diesel schema and models
├── container.rs         # composition root: the application dependencies
├── lib.rs
└── main.rs              # entrypoint: config, logging, database, HTTP server
```

## Running the application locally
To run the application locally, you need to have a Postgres database running.
You can use the `run_postgres.sh` script in the `scripts` directory to run a Postgres container.
```bash
./scripts/run_postgres.sh
```

You can then run the application.
```bash
cargo run
```

The application reads its configuration from the environment. The database connection 
string is taken from the `DATABASE_URL` environment variable (a `.env` file is 
supported as well). The server host and port default to `127.0.0.1:8080` and can be 
changed with the `SERVER_HOST` and `SERVER_PORT` environment variables.

## Testing support
All tests can be found under the `tests` folder at the crate root. When using the 
template you can place all your integration tests in this folder.

To run the tests, you can use the following command:
```bash
cargo test
```
The integration tests spin up a temporary Postgres database in a docker container, 
so no database setup is needed.
To run the tests with error output you can run the following command:
```bash
cargo test -- --nocapture
```
or 
```bash
cargo test -- --show-output
```

## Diesel ORM
The template uses Diesel ORM for its database connection and database models
integration. Its is currently setup with postgres, however you can 
change it to any other database that is supported by diesel. For other databases 
have a look at the official Diesel documentation that can be found [here](https://diesel.rs/)

### Database migrations

1) Make sure you have the diesel cli installed. You can install it with the following command:
    ```bash
    cargo install diesel_cli --no-default-features --features postgres
    ```
2) Add your postgres database url to the .env file:
    ```bash
    echo DATABASE_URL=postgres://username:password@localhost/diesel_demo > .env
    ```
3) Setup diesel before creating a migration:
    ```bash
    diesel setup
    ```
4) Create a migration with the following command:
    ```bash
    diesel migration generate <migration_name>
    ```
5) Apply your migrations:
    ```bash
    diesel migration run
    ```

## Repository and use-case pattern
The business logic of the application lives in use cases in the domain layer. 
Repositories in the data layer provide the storage access. An example use case:

```rust
// Placed under /src/domain/use_cases/todos/create_todo.rs
pub struct CreateTodoUseCase {
    todo_repository: TodoRepository,
}

#[async_trait]
impl BaseUseCase<CreateTodoRequest, CreateTodoResponse> for CreateTodoUseCase {
    async fn execute(
        &self,
        request_object: CreateTodoRequest,
    ) -> Result<CreateTodoResponse, OtherError> {
        let todo = self.todo_repository.create(&request_object).await?;
        Ok(CreateTodoResponse::from(todo))
    }
}
```

An example repository implementation:

```rust
// Placed under /src/data/todos/repository.rs
#[async_trait]
impl BaseRepository for TodoRepository {
    type Model = TodoDiesel;
    type Dto = TodoDTO;
    type Create = CreateTodoRequest;
    type Filters = TodoFilters;

    async fn create(&self, obj_data: &Self::Create) -> RepositoryResult<Self::Dto> {
        let new_row = CreateTodoDiesel { ... };
        let pool = self.pool.clone();
        let todo: TodoDiesel = run(move || -> RepositoryResult<TodoDiesel> {
            let mut conn = pool.get()?;
            Ok(diesel::insert_into(todos)
                .values(new_row)
                .get_result::<TodoDiesel>(&mut conn)?)
        })
        .await
        .map_err(RepositoryError::from)?;
        Ok(todo.into())
    }
}
```

Diesel is blocking, so the storage statements are executed on a blocking thread 
pool with `actix_threadpool::run`; the repositories stay async and the handlers 
await them.

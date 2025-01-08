# Zero to Production with Axum
Working on the book Zero 2 Production with Axum.

## Progress
- [x] Chapter 1
- [x] Chapter 2
- [x] Chapter 3
- [x] Chapter 4
- [x] Chapter 5
- [] Chapter 6

## Follow along
If you want to follow along consider this chronological list of my branches:
- [chapter_1]()
- [chapter_3]()
- [chapter_4]()
- [chapter_5_part_1]()
- [chapter_5_part_1_update_pg_version]()
- [chapter_5_part_2]()
- [chapter_5_part_2_1_fix_hostname]()
- [chapter_6]()

## Learned
### Chapter 3
#### How to add sqlx to a project
We chose `sqlx`  as our database crate. It was our preferred solution over the other options `diesel` and `tokio-postgres` due to its compile-time safety, async support, and SQL query syntax.
We added its own `[dependencies.sqlx]` entry to our `cargo.toml` file and defined all features needed.
To interact with database migrations we installed `sqlx-cli` globally via `cargo install --version="~0.7" sqlx-cli --no-default-features \ --features rustls,postgres`

#### How to add a Docker container that runs Postgres
Install Docker Desktop and run the following shell script:
```sh
#!/usr/bin/env bash
set -x
set -eo pipefail
if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi
if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 " cargo install --version='~0.7' sqlx-cli \
--no-default-features --features rustls,postgres"
  echo >&2 "to install it."
  exit 1
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

# Allow to skip Docker if a dockerized Postgres database is already running
if [[ -z "${SKIP_DOCKER}" ]]; then
  docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    -d postgres \
    postgres -N 1000
fi

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done
>&2 echo "Postgres is up and running on port ${DB_PORT} - running migrations now!"

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"
```
#### How to add migrations
The above script already includes the necessary commands to create a database (`sqlx database create`) and run migrations (`sqlx migrate run`). For that to happen, we will need a `DATABASE_URL` to be defined though.

#### Database constraints
Constraints like `UNIQUE` or `NOT NULL` come at a cost, the database has to ensure all checks pass before writing new data into the table.

>Database **constraints** are useful as a last line of defence from application bugs but they come at a cost - the database has to ensure all checks pass before writing new data into the table. Therefore constraints impact our write-throughput, i.e. the number of rows we can INSERT/UPDATE per unit of time in a table (p. 60).

#### Use the config crate to drive database settings
Next we added the config crate with which we can read our database settings from a `configuration.yaml` file. Once the database settings are represented by a Settings struct we can use the `try_deserialize` function to read the config.

#### Using PgPool over PgConnection
When we did our `sqlx query` and ran `execute` on it, the executor expected an argument that implements the Executor trait. `PgConnection` does not. Besides a `mut PgConnection` we can use a `PgPool` to handle this requirement. With that we also increase the maximum number of concurrent connections and increase resiliency.

### Chapter 4 - Telemetry

#### Why and how to handle unknown unknowns
Unknown unknowns are situations that could happen at some time in the future without us being prepared. A cluster could be damaged or the DB could go down. While we can't be prepared for every situation we can introduce collecting telemetry to get a better understanding of when and why something happened.

#### What should be logged?
Any interaction with external systems over the network should be closely monitored.

#### What logger did we choose?
We started with `log` and `env-log`.

#### What should we capture in log records?
We would need an id, a level and some additional information.

#### What does it mean to correlate all logs related to the same request?
Logs can suffer from race conditions. Imagine 1000 microservices that all write to a database. Each log should have a correlation id.

#### What is the issue we had with logging? Why are they the wrong abstraction?
Log statements are isolated events happening at a defined moment in time that we are trying to use to represent a tree-like processing pipeline. Logging does not represent asynchronous event flows properly.

#### What is the alternative?
Tracing

#### What is the difference between logging and tracing?
Logging focuses on events, tracing on flows. Logs provide high-level event information, while traces provide contextual details about how those events are related.

#### Explain futures with actix-web Executors
The executor might have to poll its future more than once to drive it to completion.

#### What is tracing and tracing-subscriber?
A simplistic view of the `tracing` crate is that it provides a set of publishers. All the macros in the `tracing` crate produce events or spans but there is no code observing the events or spans. So, we need some code to listen to the publishers which is availiable in the `tracing-subscriber` crate.

#### What is a span?
Spans represent periods of time in which a program was executing in a particular context.
A span consists of [fields](https://docs.rs/tracing/latest/tracing/field/index.html "mod tracing::field"), user-defined key-value pairs of arbitrary data that describe the context the span represents, and a set of fixed attributes that describe all `tracing` spans and events.

#### What is a Sink?
The concept of swallowing certain logs.

#### How to check for env variables being passed to the program?

#### How do you wrap a function in a span?
Using the tracing macro `instrument`

#### Why do you want to wrap a function in a span?
Because we are interested in all of the functions behavior.

### Chapter 5 - Going Live

### How are we able to use different environment files for deployment?
The approach was to layer our configuration, meaning to have a `base.yaml`, `production.yaml` and `local.yaml`. These are in a top-level folder called `configuration`.  We also have a `configuration.rs` folder that handles reading from these yaml files. Crates that come into play are: `config`, `secrecy`, and `serde`. To make sure we only allow two environments and handle errors due to typo's we created an `Environment` enum as well.
Reading the configuration file happens in its own `get_configuration` fn and uses the `config` Builder like so `config::Config::builder().add_source(...)`. Our `base.yaml` covers all settings that overlap between local and production environments. These include database settings. Both local and production environment though have their own application host though. By setting an `ENV` variable in our docker file to `APP_ENVIRONMENT production`, we are able to have our config set to our production variable. If no environment variable is set our `configuration.rs` file handles a default to `local`.

#### Why are we splitting our Dockerfile into a Builder and a Runtime stage?
Rust compile times can take long. If we split the builder stage from the runtime stage we can improve those. The builder stage does not contribute to its size and is discarded at the end. 

#### How do we make sure we are not leaking our database credentials?
We pulled in the secrecy crate and defined the password as a Secret. For the production deployment in DigitalOcean we made use of environment variables defined in our `~/spec.yaml`

#### How are we able to set ssl as required for our production deployments and not-required for our local deployments?
We make use of different `yaml` configuration files where we set a `ssl_required` flag which will later be read out

### Chapter 6 - Rejecting invalid Subscribers 
#### What does _defense in depth_ stand for?
#### How should we validate our name input?
#### What do we mean by local and global approach?
#### How do we make sure that we don't have to check form.name for validity every time it is being used?
#### How can we solve for the issue of having to repeat a is_valid(String) function everywhere?
#### What is a Tuple Struct and how does it look?
#### What are Graphemes?


### Tidbits
#### Different status code returned
Axum returns `StatusCode::UNPROCESSABLE_ENTITY` whereas actix-web `StatusCode::BAD_REQUEST` when a handler can't process the body of an incoming request.

#### `error: error with configuration: relative URL without a base` when running `sqlx migrate run`
This happened at the end of chapter 5 when we tried to migrate the new database URL. I forgot to wrap that URL in double quotes.

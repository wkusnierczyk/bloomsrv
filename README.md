# bloomsrv

**Bloom Server** (`bloomsrv`) is a high-performance, asynchronous RESTful API service that provides access to in-memory [Bloom Filters](https://en.wikipedia.org/wiki/Bloom_filter) implemented in the [`bloomlb`](https://crates.io/crates/bloomlib) library.

It allows clients to create, manage, and interact with probabilistic data structures over HTTP.
This service is ideal for distributed systems that need a lightweight, fast, and space-efficient way to check for set membership (e.g., checking if a username is taken, URL caching, or deduping streams) without maintaining a local filter instance in every client.

## Table of Contents
- [About `bloomlib`](#about-bloomlib)
- [Repository structure](#structure)
- [Dependencies](#dependencies)
- [Design and Implementation](#design-and-implementation)
- [Building and Testing](#building-and-testing)
- [Running the Service](#running-the-service)
- [API Usage Guide](#api-usage-guide)
- [Docker](#docker)

---

## About `bloomlib`

The core logic for the probabilistic data structure is provided by **bloomlib**.

* **Documentation:** [docs.rs/bloomlib](https://docs.rs/bloomlib)
* **Crates.io:** [crates.io/crates/bloomlib](https://crates.io/crates/bloomlib)
* **Source Code:** [github.com/wkusnierczyk/bloomlib](https://github.com/wkusnierczyk/bloomlib)

---

## Structure

This project follows the idiomatic Rust library plus binary pattern to separate core logic from server startup code. 
This ensures the application is easily testable and modular.

```
.
├── Cargo.toml          # Project configuration and dependencies
├── README.md           # Documentation
├── src/
│   ├── lib.rs          # Core Library: Contains models, state, and router logic
│   └── main.rs         # Binary Entrypoint: Starts the TCP listener
└── tests/
    └── api_tests.rs    # Integration Tests: Black-box HTTP tests
```

* `src/lib.rs`: The heart of the application. It defines the `FilterContainer`, the shared state, and the `create_app` function. 
It also contains unit tests (via doc-tests) to verify internal logic.

* `src/main.rs`: A thin wrapper that imports the logic from `src/lib.rs`, sets up the `tokio` runtime, and binds the server to port 3000 on the localhost.

* `tests/api_tests.rs`: Contains integration tests. These tests treat the application as a black box, spinning up a router and sending real HTTP requests to verify the full API lifecycle.

## Dependencies

This project relies on the robust Rust ecosystem for asynchronous networking and serialization.

| Crate | Description                                                                                                  | [crates.io](https://crates.io)                                  | [docs.rs](https://docs.rs/)                          | [github.com](https://github.com)                                           |
| :--- |:-------------------------------------------------------------------------------------------------------------|:----------------------------------------------------------------|:-----------------------------------------------------|:---------------------------------------------------------------------------|
| **Axum** | A modern, ergonomic web framework that routes HTTP requests to handlers.                                     | [`crates.io/axum`](https://crates.io/crates/axum)               | [`docs.rs/axum`](https://docs.rs/axum)               | [`github.com/tokio-rs/axum`](https://github.com/tokio-rs/axum)             |
| **Parking_lot** | Provides smaller, faster, and more flexible synchronization primitives (`RwLock`) than the standard library. | [`crates.io/parking_lot`](https://crates.io/crates/parking_lot) | [`docs.rs/parking_lot`](https://docs.rs/parking_lot) | [`github.com/Amanieu/parking_lot`](https://github.com/Amanieu/parking_lot) |
| **Serde** | A framework for serializing and deserializing Rust data structures efficiently.                              | [`crates.io/serde`](https://crates.io/crates/serde)             | [`docs.rs/serde`](https://docs.rs/serde)             | [`github.com/serde-rs`](https://github.com/serde-rs/serde)                 |
| **Tokio** | An asynchronous runtime providing the event loop and non-blocking I/O.                                       | [`crates.io/tokio`](https://crates.io/crates/tokio)             | [`docs.rs/tokio`](https://docs.rs/tokio)             | [`github.com/tokio-rs`](https://github.com/tokio-rs/tokio)                 |
| **Tower** | Used primarily in testing to invoke the service directly without a TCP socket.                               | [`crates.io/tower`](https://crates.io/crates/tower)             | [`docs.rs/tower`](https://docs.rs/tower)             | [`github.com/tower-rs`](https://github.com/tower-rs/tower)                 |
| **Uuid** | Generates unique 128-bit identifiers for every new filter created.                                           | [`crates.io/uuid`](https://crates.io/crates/uuid)               | [`docs.rs/uuid`](https://docs.rs/uuid)               | [`github.com/uuid-rs`](https://github.com/uuid-rs/uuid)                    |

---

## Design and Implementation

### Architecture
The application is structured as a **shared-state REST API**.

1.  **State Management:**
    The core state is stored in a `HashMap`, mapping filter names to a `FilterContainer`.
    ```rust
    type SharedState = Arc<RwLock<HashMap<String, FilterContainer>>>;
    ```
    * **`Arc` (Atomic Reference Counted):** Allows the state to be owned by multiple concurrent threads (request handlers).
    * **`RwLock` (Read-Write Lock):** Supports high-concurrency optimization. It allows multiple clients to `Lookup` (read) simultaneously, but enforces exclusive access for `Insert` or `Create` (write) operations.

2.  **Filter Container:**
    `SharedState` does not store raw filter objects. Filter instances are wrapped in a `FilterContainer` struct that additionally holds metadata (Capacity, Creation Mode, UUID). This design provides rich metadata in List responses.

3.  **Concurrency Model:**
    Powered by `Tokio`, the service is non-blocking. Heavy I/O or waiting for locks yields execution back to the runtime, allowing a single instance to handle thousands of concurrent connections efficiently.

---

## Building and Testing

### Prerequisites
* Rust (latest stable)
* Cargo

### Build

Compile the project using `cargo`.

```bash
cargo build
cargo build --release
```

### Test
The project includes Unit Tests (via Doc-tests in `lib.rs`) and Integration Tests (`tests/api_tests.rs`).

Run the full suite using `cargo`.

```bash
cargo test
```

_Example test output_

```
   Compiling bloomsrv v0.1.0 (/Users/waku/dev/bloom-service)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.47s
     Running unittests src/lib.rs (target/debug/deps/bloomsrv-afa3cedfe75d0025)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/bloomsrv-6b0170f45d9a4854)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/api_tests.rs (target/debug/deps/api_tests-9b6c3b73b9d85f7f)

running 3 tests
test test_delete_non_existent ... ok
test test_create_filter_validation ... ok
test test_full_filter_lifecycle ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests bloomsrv

running 3 tests
test src/lib.rs - FilterContainer (line 23) ... ok
test src/lib.rs - CreationMode (line 43) ... ok
test src/lib.rs - create_app (line 102) ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
```

**Note**
* There are no individual unit tests in the libary (`src/lib.rs`) and the entry point (`src/main.rs`) source files, hence the first two `runnin 0 tests` messages.
* Unit tests of the library are included within the doc-tests, hence the `running 3 tests` message under `Doc-tests bloomsrv` section.
* Tests in `tests/api_tests.rs` exercise the full API through interacting with an actually running service.

## Running the Service

Start the server using `cargo`.

```bash
# Run from sources
cargo run
```

_Example output_

```
Bloom Daemon listening on [http://127.0.0.1:3000](http://127.0.0.1:3000)
```

To download from crates.io and run the binary without building from local sources, use `cargo install`.

```bash
# Specify installation path
BIN_PATH=/usr/local 

# Install binary to specified path
cargo install bloomsrv --root "${BIN_PATH}"

# Run the installed service in the background
bloomsrv &
```

**Note** 
* By default, `bloomsrv` listens on `127.0.0.1:3000`.
* The `--host` and `--port` options allow to specify a different host and port.
* Alternatively, the `BLOOMSRV_HOST` and `BLOOMSRV_PORT` environment variables can be used.

```bash
# Specify host and port via command line options
bloomsrv --host <host> --port <port>

# Specify host and port via environment variables
BLOOMSRV_HOST=<host> BLOOMSRV_PORT=<port> bloomsrv
```

In the documentation below, the service is run with the default host and port.

---

## API Usage Guide

Below is an extensive guide to interacting with the service's REST API.
The service listens on **port 3000** by default.
At this time, the host anad port are fixed in the source code.
Future versions may support dynamic configuration.

**Note**
* The examples below use the `curl` command-line utility for the requests.
* The `jq` command-line utility was used to pretty-print the received JSON responses.

```bash
curl <request> | jq
```

* In all but the first example, the verbose output from curl is omitted (as if `curl` were called with the option `-s`).

### Create a filter

You can create a filter by specifying the estimated item count and _either_ a target false positive rate, _or_ a fixed hash count.

**Request**

|                     |            |
|:--------------------|:-----------|
| **Method**          | POST       |
| **Endpoint**        | `/filters` |
| **Body** (option 1) | `{ "name" : <filter name>, "item_count": <count>, "false_positive_rate": <rate> }`
| **Body** (option 2) | `{ "name" : <filter name>, "item_count": <count>, "hash_count": <count> }`

_Example_

```bash
curl -X POST http://127.0.0.1:3000/filters \
     -H "Content-Type: application/json" \
     -d '{
          "name": "login_attempts",
          "item_count": 1000,
          "false_positive_rate": 0.01
     }'
```

**Response**

| Outcome | Code| Body                                                                           |
|:--------|:-----|:-------------------------------------------------------------------------------|
| Success | 200 OK | `{ "id": <uuid>, "name": <filter name>, "message": "Filter created" }`         |
| Failure | 400 Bad Request | `{ "error": "Cannot create filter '<filter name>>', name is already in use" }` |

_Example_

```json
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
Dload  Upload   Total   Spent    Left  Speed
100   202  100    96  100   106  57936  63971 --:--:-- --:--:-- --:--:--  197k
{
  "id": "2d0a2947-851d-4df4-af10-5a06b4d8aad1",
  "name": "login_attempts",
  "message": "Filter created"
}
```

**Note**:
* A call to create a filter with the name of an already existing one will result in an error.

_Example_

```json
{
  "error": "Cannot create filter 'login_attempts', name is already in use"
}
```

### List all filters

List all active filters and their configurations.

**Request**

|                     |                                                                                      |
|:--------------------|:-------------------------------------------------------------------------------------|
| **Method**          | GET                                                                                  |
| **Endpoint**        | `/filters`                                                                           |
| **Body**  |  None

_Example_

```bash
curl -X GET http://127.0.0.1:3000/filters
```
**Response**

| Outcome | Code| Body                                                                           |
|:--------|:-----|:-------------------------------------------------------------------------------|
| Success | 200 OK | `{ "id": <filter uuid>, "name": <filter name>, "item_count": <count>, "config": <original parameter> }`

**Note**: 
* The `"config"` field may contain either the false positive rate or the hash count, depending on how the filter was created.
* There is no specific error code for this case, as the service maintains a list of filters at all times, even if no filter has been created yet (the list is empty).

_Example_

```json
[
  {
    "id": "2d0a2947-851d-4df4-af10-5a06b4d8aad1",
    "name": "login_attempts",
    "item_count": 1000,
    "config": "False positive rate: 0.01"
  }
]
```

### Delete a filter

Delete a specific filter by name.

**Request**

|                     |                          |
|:--------------------|:-------------------------|
| **Method**          | DELETE                   |
| **Endpoint**        | `/filters/<filter name>` |
| **Body**  | None|                     

_Example_

```bash
curl -X DELETE http://localhost:3000/filters/login_attempts
```

**Response**

| Outcome | Code| Body                                                                           |
|:--------|:-----|:-------------------------------------------------------------------------------|
| Success | 200 OK | `{ "message": "Filter '<filter name>' deleted" }` |
|Failure | 404 Not Found | `{ "error": "Filter '<filter name>' not found" }`|

_Example_

```json
{ 
  "message": "Filter 'login_attempts' deleted"
}
```

**Note**
* The implementation supports deleting a filter by UUID. However, the operation is not efficient, and is therefore not 


### Insert an item

Insert an item into to a specific filter. 

**Request**

|                     |                                |
|:--------------------|:-------------------------------|
| **Method**          | POST                           |
| **Endpoint**        | `/filters/<filter name>/items` |
| **Body**  | `<item>`                       |                     

**Note**: The request body represents the item directly. Do not wrap it in JSON.

_Example_

```bash
curl -X POST http://127.0.0.1:3000/filters/login_attempts/items \
     -d "user@example.com"
```

**Response**


| Outcome | Code| Body                                                                   |
|:--------|:-----|:-----------------------------------------------------------------------|
| Success | 200 OK | `{ "message": "Item '<item>' inserted into filter '<filter name>>'" }` |
|Failure | 404 Not Found | `{ "error": "Filter '<filter name>' not found" }`                      |

_Example_

```json
{
    "message": "Item 'user@example.com' inserted into filter 'login_attempts'"
}
```

### Test for an item in a filter

Check if an item exists in the set represented by a specific filter (has been seen by the filter).


**Request**

|                     |                                |
|:--------------------|:-------------------------------|
| **Method**          | GET                            |
| **Endpoint**        | `/filters/<filter name>/items` |
| **Body**  | `<item>`                       |                     

**Note**: The request body represents the item directly. Do not wrap it in JSON.

_Example_

```bash
curl -X GET http://127.0.0.1:3000/filters/login_attempts/items \
     -d "user@example.com"
```

**Response**

| Outcome  | Code| Body                                              |
|:---------|:-----|:--------------------------------------------------|
| Success  | 200 OK | `{ "contains": <boolean>, message": <message> }`  |
| Failure  | 404 Not Found | `{ "error": "Filter '<filter name>' not found" }` |

**Note**
* The `"contains"` field is `true` if the item may have been inserted into the filter, `false` otherwise (the item had certainly not been inserted).
* The `"message"` field provides a human-readable explanation of the result: either `"Item '<item>' may have been seen by filter '<filter name>'"` or `"Item '<item>' cannot have been seen by filter '<filter name>'"`

_Example_

```json
{
  "contains": true,
  "message": "Item 'user@example.com' may have been seen by filter 'login_attempts'"
}
```

**Note**

* The value `true` in the `"contains"` field may be misleading, as it **does not** indicate that the item has certainly been inserted into the filter.


### Clear a filter

Reset all bits in a filter to 0, effectively emptying it while keeping the configuration and ID.
After clearing and before any subsequent insertion, all item lookups will result in an `"Item '<item>' cannot have been seen by filter '<filter name>'"` response.

**Request**

|                     |                                |
|:--------------------|:-------------------------------|
| **Method**          | PUT                            |
| **Endpoint**        | `/filters/<filter name>/clear` |
| **Body**  | None                           |                     

_Example_

```bash
curl -X PUT http://localhost:3000/filters/login_attempts/clear
```

**Response**

| Outcome  | Code| Body                                                       |
|:---------|:-----|:-----------------------------------------------------------|
| Success  | 200 OK | `{ "message": "Filter '<filter name>' has been cleared" }` |
| Failure  | 404 Not Found | `{ "error": "Filter '<filter name>' not found" }`          |


_Example_

```json
{
  "message": "Filter 'login_attempts' has been cleared"
}
```

### Delete a filter

Remove a specific filter entirely from memory.

**Request**

|                     |                                |
|:--------------------|:-------------------------------|
| **Method**          | DELETE                         |
| **Endpoint**        | `/filters/<filter name>` |
| **Body**  | None                           |                     

_Example_

```bash
curl -X DELETE http://localhost:3000/filters/login_attempts
```

**Response**

| Outcome  | Code| Body                                                       |
|:---------|:-----|:-----------------------------------------------------------|
| Success  | 200 OK | `{ "message": "Filter '<filter name>' has been deleted" }` |
| Failure  | 404 Not Found | `{ "error": "Filter '<filter name>' not found" }`          |


_Example_

```json
{
  "message": "Filter 'login_attempts' has been deleted"
}
```

## Docker

The `docker/` subdirectory provides code to build a Docker image encapsulating the service.
The compiled image is available from Docker Hub as [`wkusnierczyk/bloomsrv`](https://hub.docker.com/r/wkusnierczyk/bloomsrv).
See `docker/README.md` for more details.
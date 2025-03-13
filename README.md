# Indexer Service

This service is a way to create indexers on-demand using [apibara](https://www.apibara.com/docs).

[![codecov](https://codecov.io/gh/Astraly-Labs/indexer-service/graph/badge.svg?token=3XLJIJBnzM)](https://codecov.io/gh/Astraly-Labs/indexer-service)

## Architecture

```mermaid
sequenceDiagram
    title Architecture

    actor User
    participant Indexer Service
    participant S3
    participant Database
    participant Child Process

    User->>Indexer Service: POST / {script.js}
    
    rect rgb(200, 200, 240)
        note right of User: transaction
        Indexer Service->>S3: Save script.js to S3
        S3-->>Indexer Service: ok
        Indexer Service->>Database: Create indexer in CREATED state
        Database-->>Indexer Service: ok
    end
    
    Indexer Service->>S3: get script
    S3-->>Indexer Service: script
    Indexer Service->>Child Process: start apibara indexing with binary
    Child Process-->>Indexer Service: ok
    Indexer Service->>Database: update indexer process id
    Database-->>Indexer Service: ok
    Indexer Service-->>User: ok

    alt indexer fails
        Child Process->>Indexer Service: service failed
        Indexer Service->>Database: mark indexer with id X as FailedRunning
        Database-->>Indexer Service: ok
    else indexer is stopped
        User->>Indexer Service: /stop/:id
        Indexer Service->>Database: get process id
        Database-->>Indexer Service: process id
        Indexer Service->>Child Process: terminate
        Child Process-->>Indexer Service: ok
        Indexer Service->>Database: mark as FailedStopping
        Database-->>Indexer Service: ok
        Indexer Service-->>User: ok
    else start a previously stopped indexer
        User->>Indexer Service: /start/:id
        Indexer Service->>S3: get script
        S3-->>Indexer Service: script
        Indexer Service->>Child Process: start
        Child Process-->>Indexer Service: ok
        Indexer Service->>Database: mark as Running and update process Id
        Database-->>Indexer Service: ok
        Indexer Service-->>User: ok
    end
```

## Running locally

1. Run docker compose `docker compose -f compose.dev.yaml up --build`

2. Initialize bucket `make gcs-init`

3. Create an indexer locally e.g
```bash
curl --location 'http://0.0.0.0:8081/v1/indexers' \
--form 'script.js=@"/Users/0xevolve/Documents/GitHub/indexer-service/examples/pragma/mainnet/mainnet-script-spot.js"' \
--form 'table_name="mainnet_spot_entry"' \
--form 'indexer_type="Postgres"' \
--form 'starting_block="1000000"'
```

4. Check status with
```bash
curl --location 'http://0.0.0.0:8081/v1/indexers/status/table/mainnet_spot_entry'
```

## Running tests

1. Run docker compose `docker compose -f compose.dev.yaml up --build`

2. Run tests with `cargo nextest run --test-threads=1`
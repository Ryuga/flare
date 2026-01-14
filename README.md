# Flare

Flare is a high-performance, lightweight distributed object storage engine written in Rust.


## Architecture Overview

Flare is split into **two independent services**:

```
Client
  ↓
API Node
  ├── object-level API
  ├── streaming chunker
  ├── placement logic
  ├── metadata store
  ↓
Data Nodes
  ├── chunk storage
  ├── disk IO
  ├── dumb (no understanding of objects)
  └── fast
```

### API Node

* Accepts object uploads and downloads
* Splits objects into fixed-size chunks
* Decides which data node stores each chunk
* Maintains object metadata
* Streams data end-to-end (no buffering)

### Data Node
* Stores chunks on local disk
* Serves chunks over HTTP
* Does **not** understand objects (dumb)
* Stateless 
---

## Design Principles

* **Streaming first**
  Objects are never fully buffered in memory for either uploads or downloads.

* **Separation of responsibilities**
  API nodes orchestrate. Data nodes store bytes.

* **Deterministic correctness**
  No hidden assumptions about ordering or storage. Chunk order and placement are recorded, not inferred.


## Current Features

* Streaming `PUT /object/{key}`
* Streaming `GET /object/{key}`
* Fixed-size chunking
* Multi-node chunk distribution
* Explicit chunk ordering
* Metadata-backed reads
* Byte-for-byte correctness (verified with binary files)
---

## Building the Project

### Prerequisites

* Rust (stable)
* Cargo

#### Build everything

```bash
cargo build
```

#### Build specific services

```bash
cargo build --bin api
cargo build --bin datanode
```
---

## Running Flare

### Start a data node

```bash
cargo run --bin datanode
```

#### Default port: `9000`

You can start multiple data nodes on different ports. Make sure to link them inside `api`

### Start the API node

```bash
cargo run --bin api
```
#### Default port: `8000`

---

## Using the API (Curl examples)

### Upload an object (raw binary)

⚠️ **Must use raw binary (multipart not supported yet)*

```bash
curl -X PUT \
  --data-binary @file.zip \
  http://localhost:8000/object/my-object
```

### Download an object

```bash
curl http://localhost:8000/object/my-object > out.zip
```

Verify correctness:

```bash
cmp file.zip out.zip
```

---

## Known Limitations

* Flare stores exactly the bytes it receives
* Multipart/form-data uploads are not supported
* Metadata is currently in-memory and lost on restart
* Data nodes are assumed to be reachable and trusted

## Current Data Distribution

* Chunks are distributed round-robin across data nodes
* Placement decisions are recorded in metadata
* Reads rely entirely on metadata for correctness


---

## Planned Features

The following features are planned and will be implemented step by step:

* **Consistent hashing for placement**
  Replace round-robin distribution with deterministic placement for scaling nodes.

* **Per-chunk checksums**
  Detect data corruption early and verify integrity during reads.

* **Parallel chunk reads**
  Improve read performance by fetching chunks concurrently while preserving correct byte order.

* **Replication**
  Store multiple copies of each chunk across different data nodes

* **Persistent metadata store**
  Move metadata out of memory into db layer.

* **Consensus-backed metadata (Raft)**
  Ensuring consistency and availability of metadata across API nodes.

* **Rebalancing and healing**
  Automatically redistribute data when nodes are added, removed, or fail.

* **Multipart uploads**
  Support form-based and multipart upload workflows.

* **Observability and metrics**
  Add logging, metrics, and monitoring.




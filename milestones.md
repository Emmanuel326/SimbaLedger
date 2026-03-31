cat > MILESTONE.md << 'EOF'
# SimbaLedger Roadmap: From Working Prototype to TigerBeetle Performance

## Current Status (As of April 1, 2026)

✅ Completed:
- Core data structures (Account, Transfer, Flags)
- Double-entry accounting engine
- In-memory storage
- TCP server with JSON API
- Idempotency (durable)
- Atomic transfers (all-or-nothing)
- LSM tree persistent storage
- Basic demo mode

---

## Phase 1: Crash Safety & Durability (Week 1)

### 1.1 Write-Ahead Log (WAL)
- [ ] Implement WAL before applying state changes
- [ ] WAL entries: transfer ID, account IDs, amounts, operation type
- [ ] fsync on every commit (configurable batch size)
- [ ] Recovery: replay WAL on startup
- [ ] Test: kill -9 during write, verify recovery

### 1.2 Checksums
- [ ] Add CRC32/xxHash to WAL entries
- [ ] Checksum on LSM blocks
- [ ] Detect corruption on read
- [ ] Test: manually corrupt a byte, verify detection

### 1.3 Graceful Shutdown
- [ ] Signal handling (SIGTERM, SIGINT)
- [ ] Flush pending writes
- [ ] Close LSM trees cleanly

---

## Phase 2: Batch Protocol & Throughput (Week 2)

### 2.1 Binary Batch Protocol
- [ ] Replace JSON with binary format (MessagePack or custom)
- [ ] Batch size: 8190 transfers per call (TigerBeetle standard)
- [ ] Protocol: [length][batch_id][num_transfers][transfer1...]
- [ ] Implement ProtocolCodec trait with multiple implementations

### 2.2 Linked Events (Batch Atomicity)
- [ ] Add `linked` flag to transfers
- [ ] Batch validation: all or nothing
- [ ] Rollback on any failure
- [ ] Test: partial failure in batch

### 2.3 Batching in Engine
- [ ] Modify engine to accept batches, not single transfers
- [ ] Process batch in one transaction
- [ ] One network call = one batch

### 2.4 Performance Benchmarking
- [ ] Benchmark tool: multi-threaded client
- [ ] Measure TPS on ThinkPad
- [ ] Target: 5,000-8,000 TPS (single-threaded)

---

## Phase 3: Kernel Bypass & Zero-Copy (Week 3)

### 3.1 io_uring Integration
- [ ] Replace std::net with tokio-uring or io-uring crate
- [ ] Use io_uring for network I/O
- [ ] Use io_uring for disk I/O (direct I/O, O_DIRECT)
- [ ] Single-threaded reactor pattern

### 3.2 Zero-Copy
- [ ] Receive batch directly into pre-allocated buffer
- [ ] Parse without copying
- [ ] Write to disk without copying
- [ ] Use `sendmsg`/`recvmsg` with scatter-gather

### 3.3 Memory Pooling
- [ ] Pre-allocate all structures at startup
- [ ] No allocation after startup
- [ ] Use `arrayvec` for fixed-size vectors
- [ ] Bump allocator for temporary buffers

### 3.4 CPU Affinity
- [ ] Pin thread to one core
- [ ] Disable interrupts on that core (if possible)
- [ ] Measure cache misses

### 3.5 Performance Target
- [ ] Benchmark: 10,000-15,000 TPS on ThinkPad T470s

---

## Phase 4: Distributed Consensus (Week 4-5)

### 4.1 Raft Implementation
- [ ] Add Raft consensus (using `raft-rs` or custom)
- [ ] Replication to 3-6 nodes
- [ ] Leader election
- [ ] Log replication
- [ ] Snapshotting

### 4.2 Replication Trait
- [ ] Define Replication trait
- [ ] Single-node implementation (current)
- [ ] Raft implementation
- [ ] VSR implementation (future)

### 4.3 Multi-Node Testing
- [ ] Local cluster testing (multiple processes)
- [ ] Network partition simulation
- [ ] Leader failover
- [ ] Split-brain prevention

### 4.4 Jepsen-style Testing
- [ ] Use `async-maelstrom` for Jepsen tests
- [ ] Run against 3-node cluster
- [ ] Verify linearizability
- [ ] Test: network partitions, node kills, clock skew

---

## Phase 5: Advanced Performance (Week 6)

### 5.1 LSM Tuning
- [ ] Tune compaction strategy (leveled vs tiered)
- [ ] Block size optimization (4KB, 16KB)
- [ ] Bloom filters for point lookups
- [ ] Compression (LZ4, zstd)

### 5.2 Read Path Optimization
- [ ] Batched reads for batch verification
- [ ] Prefetch accounts in batch
- [ ] Account cache (LRU)

### 5.3 CPU Profiling
- [ ] perf/flamegraph profiling
- [ ] Identify bottlenecks
- [ ] Optimize hot paths
- [ ] Reduce branch mispredictions

### 5.4 Performance Target
- [ ] 15,000-20,000 TPS on ThinkPad T470s
- [ ] 42,000+ TPS on modern hardware (M1 Max)

---

## Phase 6: Production Hardening (Week 7)

### 6.1 Observability
- [ ] Metrics: Prometheus exporter
- [ ] Traces: OpenTelemetry
- [ ] Logs: structured (JSON)
- [ ] Dashboard: Grafana

### 6.2 Configuration
- [ ] Config file (TOML/YAML)
- [ ] Environment variables
- [ ] Command-line flags

### 6.3 Security
- [ ] TLS support
- [ ] Authentication (JWT or mTLS)
- [ ] Authorization (API keys)
- [ ] Rate limiting

### 6.4 Deployment
- [ ] Docker container
- [ ] Kubernetes Helm chart
- [ ] Systemd service
- [ ] Backup/restore procedures

---

## Phase 7: The Bellard Demo (Week 8)

### 7.1 Demo Environment
- [ ] Single ThinkPad T470s (i5-6300U)
- [ ] 7-day continuous run
- [ ] Simulate 100M transactions (M-PESA weekly volume)

### 7.2 Benchmark Tools
- [ ] Load generator: simulate M-PESA traffic pattern
- [ ] Zipfian distribution (hot accounts)
- [ ] Realistic batch sizes
- [ ] Monitor: TPS, latency (p50, p95, p99), disk I/O, CPU

### 7.3 Comparison Data
- [ ] Safaricom's published numbers: 4,000 TPS (current), 6,000 TPS (target)
- [ ] SimbaLedger results: 15,000+ TPS
- [ ] Cost comparison: 1 laptop vs 700 servers

### 7.4 Demo Video
- [ ] Screen recording: htop showing 100% CPU
- [ ] Grafana dashboard: 15k TPS sustained
- [ ] Laptop power draw: 15W
- [ ] Datacenter power draw: thousands of watts

### 7.5 Whitepaper
- [ ] Architecture overview
- [ ] Benchmark methodology
- [ ] Results and analysis
- [ ] Invariants and correctness proofs

---

## Phase 8: Market Entry (Week 9-10)

### 8.1 Safaricom Pitch
- [ ] Prepare executive summary
- [ ] Demo video with ThinkPad
- [ ] TCO analysis: $10M+ annual savings
- [ ] Migration path from legacy systems

### 8.2 Open Source Release
- [ ] Choose license (MIT/Apache 2.0)
- [ ] API documentation
- [ ] Tutorials and examples
- [ ] Community guidelines

### 8.3 Competitor Analysis
- [ ] Compare with TigerBeetle
- [ ] Compare with traditional databases
- [ ] Highlight unique value: composability, Rust ecosystem, Africa-first

### 8.4 Funding Preparation
- [ ] Pitch deck
- [ ] Financial projections
- [ ] Team (you + ?)
- [ ] Investor outreach

---

## Success Criteria

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| TPS (ThinkPad T470s) | ~500 | 15,000+ | 🔄 In Progress |
| Latency p99 | 10ms | < 1ms | 🔄 |
| Crash recovery | ❌ | ✅ | 🔄 |
| Checksums | ❌ | ✅ | 🔄 |
| Batch size | 1 | 8190 | 🔄 |
| Consensus | None | Raft | 📅 |
| Jepsen-tested | ❌ | ✅ | 📅 |
| Production-ready | ❌ | ✅ | 📅 |

---

## The Timeline


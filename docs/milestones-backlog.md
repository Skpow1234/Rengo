# ShardForge milestone backlog

Generated from a full-repo code audit (TODOs, stubs, docs vs code, SQL/Raft/storage/network/CI gaps) plus granular module improvements.

**Count: 852 milestones**

Status legend: unchecked = open. Items recently implemented in the composition-root effort may still appear here until triaged.

See also: [milestones.md](milestones.md) (summary), [roadmap.md](roadmap.md) (phases).

---

## Recently completed (composition root)

These were delivered in the single-node wiring pass; kept here for traceability.

1. [x] Ship DatabaseNode composition root wiring config, storage, SQL, and transactions
2. [x] Bind TCP SQL listener from DatabaseNode::start on configured address
3. [x] Serve newline-delimited JSON SQL protocol on the node bind port
4. [x] Wire CLI `shardforge start` to open config and call DatabaseNode::start
5. [x] Wire CLI `shardforge sql` to execute against an in-process DatabaseNode
6. [x] Wire CLI `shardforge status` to report storage engine and table count from config
7. [x] Persist table schemas under __catalog__table__* keys as JSON
8. [x] Persist catalog table name list under __catalog__tables
9. [x] Reload durable catalog on DatabaseNode open
10. [x] Extract PRIMARY KEY columns from CREATE TABLE constraints into TableSchema
11. [x] Allocate INSERT row ids via durable __seq__<table> counters
12. [x] Replace 0..1000 key probing with StorageEngine::scan_prefix
13. [x] Implement scan_prefix for MemoryEngine / SledEngine / RocksDBEngine
14. [x] Delete table row data on DROP TABLE via prefix scan
15. [x] Auto-begin/commit TransactionManager around SQL statements in DatabaseNode
16. [x] Attach DatabaseNode to DatabaseServiceImpl for execute_query
17. [x] Register [[test]] targets so integration tests are discovered
18. [x] Add node_integration tests for DDL/DML through DatabaseNode

---

## Open backlog



1. [ ] [Raft] Implement start_election on election timeout
2. [ ] [Raft] Candidate RequestVote to all peers
3. [ ] [Raft] Become-leader transition and LeaderState init
4. [ ] [Raft] Leader heartbeat AppendEntries loop
5. [ ] [Raft] Log replication pipeline with flow control
6. [ ] [Raft] Advance commit_index from match_index quorum
7. [ ] [Raft] Apply committed entries to state machine loop
8. [ ] [Raft] Durable RaftStorage on Sled
9. [ ] [Raft] Durable RaftStorage on RocksDB
10. [ ] [Raft] TCP transport for RequestVote
11. [ ] [Raft] TCP transport for AppendEntries
12. [ ] [Raft] TCP transport for InstallSnapshot
13. [ ] [Raft] Wire ConnectionPool into Raft peer RPCs
14. [ ] [Raft] Finish pre-vote protocol end-to-end
15. [ ] [Raft] Joint consensus membership changes
16. [ ] [Raft] Populate snapshot configuration field
17. [ ] [Raft] Chunked InstallSnapshot transfer
18. [ ] [Raft] Install snapshot into storage engine
19. [ ] [Raft] Log compaction after successful snapshot
20. [ ] [Raft] Leader transfer API
21. [ ] [Raft] Read-index / lease reads
22. [ ] [Raft] Observer role support
23. [ ] [Raft] Witness role support
24. [ ] [Raft] Persist voted_for/term with fsync
25. [ ] [Raft] Raft election/replication/commit Prometheus metrics
26. [ ] [Raft] Property tests for AppendEntries conflicts
27. [ ] [Raft] Propose SQL writes through Raft when clustered
28. [ ] [Raft] Raft peers list in shardforge.toml
29. [ ] [Raft] Expose Raft role in CLI status
30. [ ] [Raft] Randomized election timeout jitter
31. [ ] [Raft] Cancel election timer on valid heartbeat
32. [ ] [Raft] Snapshot restore on node boot
33. [ ] [Raft] Reject stale terms in all RPC handlers
34. [ ] [Raft] Learner catch-up throttling
35. [ ] [Raft] Network partition recovery tests
36. [ ] [Raft] 3-node docker-compose Raft quorum smoke test
37. [ ] [Raft] 5-node membership add/remove test
38. [ ] [Raft] Raft log truncation safety tests
39. [ ] [Raft] Serialize Raft RPC with versioned codec
40. [ ] [Raft] Backpressure when replication lag high
41. [ ] [Raft] Follower read redirection to leader
42. [ ] [Network-Security] Real tonic::Server::serve with grpc-enabled
43. [ ] [Network-Security] Codegen proto module path stable
44. [ ] [Network-Security] Fail build when protoc missing if feature on
45. [ ] [Network-Security] Implement ExecuteBatch RPC
46. [ ] [Network-Security] Implement StreamQuery RPC
47. [ ] [Network-Security] Implement JoinCluster streaming
48. [ ] [Network-Security] Implement LeaveCluster
49. [ ] [Network-Security] Implement Heartbeat with cluster updates
50. [ ] [Network-Security] Implement GetClusterTopology live
51. [ ] [Network-Security] Implement PrepareTransaction RPC
52. [ ] [Network-Security] Implement CommitTransaction RPC
53. [ ] [Network-Security] Implement RollbackTransaction RPC
54. [ ] [Network-Security] Implement GetMetrics streaming
55. [ ] [Network-Security] Implement ExecuteAdminCommand
56. [ ] [Network-Security] Implement GetSchema RPC
57. [ ] [Network-Security] Implement UpdateSchema RPC
58. [ ] [Network-Security] Implement GetShardMap RPC
59. [ ] [Network-Security] TLS from network.tls_* config
60. [ ] [Network-Security] mTLS with CA path
61. [ ] [Network-Security] gRPC auth middleware
62. [ ] [Network-Security] Per-RPC authorization
63. [ ] [Network-Security] Enforce max_connections
64. [ ] [Network-Security] Request deadlines/cancellation
65. [ ] [Network-Security] Client RPC timeouts
66. [ ] [Network-Security] Connection pool health checks
67. [ ] [Network-Security] Wire LoadBalancer into client
68. [ ] [Network-Security] Separate SQL and metrics ports clearly
69. [ ] [Network-Security] HTTP readiness endpoint
70. [ ] [Network-Security] HTTP liveness endpoint
71. [ ] [Network-Security] Audit log sink
72. [ ] [Network-Security] Redact secrets in logs
73. [ ] [Network-Security] SIGTERM graceful drain
74. [ ] [Network-Security] Max message size enforcement
75. [ ] [Network-Security] Raft peer authentication
76. [ ] [Network-Security] Rotate Grafana default password
77. [ ] [Network-Security] Document distroless non-root ops
78. [ ] [Network-Security] Use advertised_address in membership
79. [ ] [Network-Security] Rate limit admin RPCs
80. [ ] [Network-Security] gRPC reflection in dev builds
81. [ ] [SQL-Executor] Enforce PRIMARY KEY uniqueness
82. [ ] [SQL-Executor] Enforce UNIQUE constraints
83. [ ] [SQL-Executor] Enforce CHECK constraints
84. [ ] [SQL-Executor] Enforce FOREIGN KEY on insert
85. [ ] [SQL-Executor] Enforce FOREIGN KEY on delete
86. [ ] [SQL-Executor] Apply DEFAULT when INSERT omits columns
87. [ ] [SQL-Executor] AUTO_INCREMENT / serial sequences
88. [ ] [SQL-Executor] HAVING evaluation
89. [ ] [SQL-Executor] Complex SELECT expressions
90. [ ] [SQL-Executor] LIKE pattern matching
91. [ ] [SQL-Executor] IN list evaluation completeness
92. [ ] [SQL-Executor] IS NULL / IS NOT NULL completeness
93. [ ] [SQL-Executor] Nested WHERE boolean trees
94. [ ] [SQL-Executor] JOIN execution nested loop
95. [ ] [SQL-Executor] JOIN hash join
96. [ ] [SQL-Executor] JOIN merge join
97. [ ] [SQL-Executor] Subquery execution
98. [ ] [SQL-Executor] CTE materialization
99. [ ] [SQL-Executor] UNION execution
100. [ ] [SQL-Executor] WINDOW function runtime
101. [ ] [SQL-Executor] CASE expression runtime
102. [ ] [SQL-Executor] INSERTâ€¦SELECT runtime
103. [ ] [SQL-Executor] RETURNING clause
104. [ ] [SQL-Executor] EXPLAIN plan text
105. [ ] [SQL-Executor] TRUNCATE implementation
106. [ ] [SQL-Executor] VIEW expand on query
107. [ ] [SQL-Executor] Prepared AST parameter binding
108. [ ] [SQL-Executor] Named parameters
109. [ ] [SQL-Executor] Type coercion framework
110. [ ] [SQL-Executor] CAST support
111. [ ] [SQL-Executor] DECIMAL arithmetic
112. [ ] [SQL-Executor] DATE/TIME functions
113. [ ] [SQL-Executor] JSON operators
114. [ ] [SQL-Executor] UUID functions
115. [ ] [SQL-Executor] Stream large SELECT results
116. [ ] [SQL-Executor] Batch INSERT via batch_write
117. [ ] [SQL-Executor] Index build on CREATE INDEX from existing rows
118. [ ] [SQL-Executor] ALTER TABLE ADD CONSTRAINT
119. [ ] [SQL-Executor] ALTER TABLE DROP COLUMN
120. [ ] [SQL-Executor] ALTER TABLE ALTER COLUMN
121. [ ] [SQL-Executor] DROP TABLE CASCADE semantics
122. [ ] [SQL-Executor] DROP TABLE RESTRICT semantics
123. [ ] [SQL-Executor] Typed binary row format
124. [ ] [SQL-Executor] Statistics refresh after DML
125. [ ] [SQL-Executor] Histogram-backed selectivity
126. [ ] [SQL-Executor] Wire planner IndexScan path
127. [ ] [SQL-Executor] Wire query cache into hot path
128. [ ] [SQL-Executor] Parallel scan within node
129. [ ] [SQL-Executor] Cost-based join ordering
130. [ ] [SQL-Executor] DISTINCT execution
131. [ ] [SQL-Executor] GROUP BY non-column expressions
132. [ ] [DX-Docs-Quality] Unify dual MVCC modules
133. [ ] [DX-Docs-Quality] Unify TransactionHandle vs TransactionManager
134. [ ] [DX-Docs-Quality] Unify NodeId UUID vs Raft u64
135. [ ] [DX-Docs-Quality] Unify IsolationLevel enums
136. [ ] [DX-Docs-Quality] Remove unused opentelemetry deps or wire them
137. [ ] [DX-Docs-Quality] Remove unused proptest/mockall/fake/rstest or use them
138. [ ] [DX-Docs-Quality] Remove clap-bin unused dep
139. [ ] [DX-Docs-Quality] Fix duplicate [dev-dependencies] in Cargo.toml
140. [ ] [DX-Docs-Quality] Align bincode 1.3 vs workspace 2.0
141. [ ] [DX-Docs-Quality] Replace production unwrap in config loader paths
142. [ ] [DX-Docs-Quality] Replace snapshot SystemTime unwrap
143. [ ] [DX-Docs-Quality] Harden iterator end_key unwrap
144. [ ] [DX-Docs-Quality] BUILD_INFO from git hash via build.rs
145. [ ] [DX-Docs-Quality] Enforce missing_docs in CI
146. [ ] [DX-Docs-Quality] Empty test_module_imports removal
147. [ ] [DX-Docs-Quality] Contrib guide
148. [ ] [DX-Docs-Quality] ISSUE templates for milestones
149. [ ] [DX-Docs-Quality] Single source of truth for roadmap vs milestones
150. [ ] [DX-Docs-Quality] Example Rust gRPC client
151. [ ] [DX-Docs-Quality] Example Python client
152. [ ] [DX-Docs-Quality] cargo xtask for proto/docker/cluster
153. [ ] [DX-Docs-Quality] sccache or cargo-chef docker builds
154. [ ] [DX-Docs-Quality] Align README badges with reality
155. [ ] [DX-Docs-Quality] Remove overclaimed Year-1-complete checkboxes
156. [ ] [DX-Docs-Quality] Document Phase honesty in overview.md
157. [ ] [DX-Docs-Quality] Risk mitigations evidence links
158. [ ] [DX-Docs-Quality] API versioning policy for database.proto
159. [ ] [DX-Docs-Quality] CHANGELOG.md generation
160. [ ] [Storage] Add transactional begin/commit/rollback on StorageEngine trait
161. [ ] [Storage] Persist MVCC versions durably
162. [ ] [Storage] MVCC GC watermark scheduler
163. [ ] [Storage] Apply compression options to RocksDB
164. [ ] [Storage] Apply block cache size to RocksDB
165. [ ] [Storage] Apply write buffer settings to RocksDB
166. [ ] [Storage] Real RocksDB stats()
167. [ ] [Storage] Real Sled stats()
168. [ ] [Storage] Accurate MemoryEngine concurrent stats
169. [ ] [Storage] Persist IndexManager to disk
170. [ ] [Storage] Fix IndexManager::get_index returning stored index
171. [ ] [Storage] GIN index implementation
172. [ ] [Storage] GiST index implementation
173. [ ] [Storage] Bloom filters for point lookups
174. [ ] [Storage] CRC/checksum validation for values
175. [ ] [Storage] Storage backup snapshot export
176. [ ] [Storage] RocksDB SST ingest bulk load
177. [ ] [Storage] Pluggable custom engine registration
178. [ ] [Storage] Align default engine across toml/features/docs
179. [ ] [Storage] At-rest encryption hooks
180. [ ] [Storage] compact(range) API
181. [ ] [Storage] MemoryEngine optional WAL
182. [ ] [Storage] Multi-column primary key encoding
183. [ ] [Storage] Tombstone/TTL support
184. [ ] [Storage] Prefix extractor for RocksDB
185. [ ] [Storage] Sled tree-per-table option
186. [ ] [Storage] Iterator reuse without full materialization
187. [ ] [Storage] Async offload for blocking RocksDB calls
188. [ ] [Storage] Storage corruption detection on open
189. [ ] [Storage] Checksum mismatch repair tool
190. [ ] [Sharding-Distributed] Hash sharding with virtual nodes
191. [ ] [Sharding-Distributed] Shard metadata service
192. [ ] [Sharding-Distributed] Online shard split
193. [ ] [Sharding-Distributed] Online shard merge
194. [ ] [Sharding-Distributed] Rebalancing planner
195. [ ] [Sharding-Distributed] Cross-shard query coordinator
196. [ ] [Sharding-Distributed] Data migration jobs
197. [ ] [Sharding-Distributed] Consistency during move
198. [ ] [Sharding-Distributed] 2PC coordinator
199. [ ] [Sharding-Distributed] 2PC participant
200. [ ] [Sharding-Distributed] Distributed deadlock detector
201. [ ] [Sharding-Distributed] Read committed distributed isolation
202. [ ] [Sharding-Distributed] Distributed query planner
203. [ ] [Sharding-Distributed] Distributed join algorithms
204. [ ] [Sharding-Distributed] Result aggregation merger
205. [ ] [Sharding-Distributed] Parallel query fan-out
206. [ ] [Sharding-Distributed] Shard map RPC population
207. [ ] [Sharding-Distributed] Range sharding strategy
208. [ ] [Sharding-Distributed] Directory sharding strategy
209. [ ] [Sharding-Distributed] Hybrid sharding strategy
210. [ ] [Sharding-Distributed] Geo shard placement hints
211. [ ] [Sharding-Distributed] Cross-region replication
212. [ ] [Sharding-Distributed] WAN latency hiding
213. [ ] [Sharding-Distributed] Global transaction coordinator
214. [ ] [Enterprise] RBAC roles and grants
215. [ ] [Enterprise] TLS required mode
216. [ ] [Enterprise] Audit logging compliance export
217. [ ] [Enterprise] Encryption at rest keys
218. [ ] [Enterprise] Key rotation
219. [ ] [Enterprise] Incremental backup
220. [ ] [Enterprise] Point-in-time recovery
221. [ ] [Enterprise] Backup verification
222. [ ] [Enterprise] Cross-region backup copy
223. [ ] [Enterprise] DR runbook automation
224. [ ] [Enterprise] SOC2 control mapping docs
225. [ ] [Enterprise] HSM integration hooks
226. [ ] [Enterprise] MFA for admin CLI
227. [ ] [Enterprise] JWT auth provider
228. [ ] [Enterprise] OAuth provider
229. [ ] [Enterprise] Kerberos provider
230. [ ] [Enterprise] Kubernetes operator CRDs
231. [ ] [Enterprise] Helm chart
232. [ ] [Enterprise] Multi-cloud terraform modules
233. [ ] [Enterprise] Plugin architecture
234. [ ] [Enterprise] Columnar analytics engine
235. [ ] [Enterprise] Materialized views
236. [ ] [Enterprise] Stream processing hooks
237. [ ] [Enterprise] ML-assisted optimizer hooks
238. [ ] [CLI-API] Interactive SQL REPL default for sql without query
239. [ ] [CLI-API] Implement shardforge-cli init for real
240. [ ] [CLI-API] Implement shardforge-cli start for real
241. [ ] [CLI-API] Implement shardforge-cli stop with PID file
242. [ ] [CLI-API] Implement shardforge-cli status against live node
243. [ ] [CLI-API] cluster init/bootstrap command
244. [ ] [CLI-API] cluster join command
245. [ ] [CLI-API] cluster status command
246. [ ] [CLI-API] config set/get commands
247. [ ] [CLI-API] dump/restore commands
248. [ ] [CLI-API] bench CLI wrapper
249. [ ] [CLI-API] migrate schema command
250. [ ] [CLI-API] Connection URI parsing
251. [ ] [CLI-API] CLI TLS flags
252. [ ] [CLI-API] SQL output json|table|csv
253. [ ] [CLI-API] Parquet import/export
254. [ ] [CLI-API] Avro import/export
255. [ ] [CLI-API] Progress bars for large imports
256. [ ] [CLI-API] doctor/diagnose command
257. [ ] [CLI-API] Unify dual CLI binaries
258. [ ] [CLI-API] Add clap shell completions
259. [ ] [CLI-API] Generate man pages
260. [ ] [CLI-API] Non-zero exit on SQL errors
261. [ ] [CLI-API] status --watch
262. [ ] [CLI-API] Admin RPC client commands
263. [ ] [CLI-API] REPL \d introspection
264. [ ] [CLI-API] REPL transaction meta-commands
265. [ ] [CLI-API] Secure REPL history
266. [ ] [CLI-API] Multi-statement script from file
267. [ ] [CLI-API] start --wait-ready
268. [ ] [CLI-API] Put shardforge-cli in workspace members
269. [ ] [CLI-API] Single authoritative shardforge binary
270. [ ] [CLI-API] JSON status output
271. [ ] [CLI-API] Quiet/verbose global flags
272. [ ] [SQL-Parser] Parse UNIQUE column attribute
273. [ ] [SQL-Parser] Parse BEGIN isolation level
274. [ ] [SQL-Parser] Parse ON DELETE referential actions
275. [ ] [SQL-Parser] Parse ON UPDATE referential actions
276. [ ] [SQL-Parser] Handle NOT IN negation
277. [ ] [SQL-Parser] Parse schema-qualified names
278. [ ] [SQL-Parser] Parse DISTINCT ON
279. [ ] [SQL-Parser] Parse LIMIT/OFFSET fully
280. [ ] [SQL-Parser] Parse ORDER BY NULLS FIRST/LAST
281. [ ] [SQL-Parser] Parse INSERTâ€¦SELECT
282. [ ] [SQL-Parser] Parse UPDATE/DELETE RETURNING
283. [ ] [SQL-Parser] Parse WITH / CTE
284. [ ] [SQL-Parser] Parse WINDOW functions
285. [ ] [SQL-Parser] Parse CASE expressions end-to-end
286. [ ] [SQL-Parser] Parse UNION/INTERSECT/EXCEPT
287. [ ] [SQL-Parser] Parse subqueries in FROM
288. [ ] [SQL-Parser] Parse subqueries in WHERE
289. [ ] [SQL-Parser] Parse JOIN â€¦ ON
290. [ ] [SQL-Parser] Parse JOIN â€¦ USING
291. [ ] [SQL-Parser] Parse NATURAL JOIN
292. [ ] [SQL-Parser] Parse LEFT/RIGHT/FULL OUTER JOIN
293. [ ] [SQL-Parser] Parse CROSS JOIN
294. [ ] [SQL-Parser] Parse CREATE VIEW
295. [ ] [SQL-Parser] Parse DROP VIEW
296. [ ] [SQL-Parser] Parse CREATE SCHEMA
297. [ ] [SQL-Parser] Parse DROP SCHEMA
298. [ ] [SQL-Parser] Parse CREATE SEQUENCE
299. [ ] [SQL-Parser] Parse GRANT/REVOKE
300. [ ] [SQL-Parser] Parse TRUNCATE
301. [ ] [SQL-Parser] Parse EXPLAIN
302. [ ] [SQL-Parser] Parse EXPLAIN ANALYZE
303. [ ] [SQL-Parser] Parse COPY FROM/TO
304. [ ] [SQL-Parser] Parse prepared statement EXECUTE
305. [ ] [SQL-Parser] Parse DEALLOCATE
306. [ ] [SQL-Parser] Parse SET session variables
307. [ ] [SQL-Parser] Parse SHOW commands
308. [ ] [SQL-Parser] Parser error spans with line/column
309. [ ] [SQL-Parser] Fuzz SqlParser with proptest
310. [ ] [SQL-Parser] Golden-file parser tests for PG samples
311. [ ] [SQL-Parser] Reject dangerous unterminated strings safely
312. [ ] [CI-Docker] Collapse duplicate CI workflows
313. [ ] [CI-Docker] Clippy -D warnings must fail CI
314. [ ] [CI-Docker] Bench failures must fail CI
315. [ ] [CI-Docker] Run discovered integration tests in CI
316. [ ] [CI-Docker] Add cargo deny check job
317. [ ] [CI-Docker] Re-enable cargo-audit
318. [ ] [CI-Docker] Coverage job with llvm-cov
319. [ ] [CI-Docker] grpc-enabled CI feature job
320. [ ] [CI-Docker] Multi-arch docker amd64/arm64
321. [ ] [CI-Docker] SBOM on release images
322. [ ] [CI-Docker] Sign container images
323. [ ] [CI-Docker] Smoke test docker start + SQL
324. [ ] [CI-Docker] Fail CI if container exits immediately
325. [ ] [CI-Docker] Add .dockerignore and verify
326. [ ] [CI-Docker] HEALTHCHECK beyond --version
327. [ ] [CI-Docker] Honor mounted config in entrypoint
328. [ ] [CI-Docker] Pin Prometheus/Grafana digests
329. [ ] [CI-Docker] Package .deb/.rpm
330. [ ] [CI-Docker] Windows installer artifact
331. [ ] [CI-Docker] Homebrew formula
332. [ ] [CI-Docker] Always ship Cargo.lock in image
333. [ ] [CI-Docker] shellcheck scripts in CI
334. [ ] [CI-Docker] pwsh script lint
335. [ ] [CI-Docker] Example TLS compose profile
336. [ ] [CI-Docker] Cluster profile forms Raft quorum test
337. [ ] [CI-Docker] Remove soft-fail || echo patterns
338. [ ] [CI-Docker] Cache protoc on Windows CI
339. [ ] [CI-Docker] docs deploy real job not echo
340. [ ] [CI-Docker] Fix Dockerfile dummy main layer smell
341. [ ] [CI-Docker] Align rust-version Dockerfile vs Cargo.toml
342. [ ] [CI-Docker] Non-root USER in runtime image
343. [ ] [CI-Docker] Read-only rootfs compatible layout
344. [ ] [Observability-Perf] Prometheus /metrics HTTP server from monitoring.bind_address
345. [ ] [Observability-Perf] QPS counter metric
346. [ ] [Observability-Perf] Latency histogram metric
347. [ ] [Observability-Perf] Raft lag gauge
348. [ ] [Observability-Perf] Storage size gauge
349. [ ] [Observability-Perf] OpenTelemetry when tracing_enabled
350. [ ] [Observability-Perf] Grafana dashboards as code
351. [ ] [Observability-Perf] pprof continuous profiling hook
352. [ ] [Observability-Perf] JSON logging from config format
353. [ ] [Observability-Perf] Log file rotation from max_file_size_mb
354. [ ] [Observability-Perf] Criterion benches for SQL parse
355. [ ] [Observability-Perf] Criterion benches for SQL execute
356. [ ] [Observability-Perf] Sled/RocksDB criterion benches
357. [ ] [Observability-Perf] Register [[bench]] harness false
358. [ ] [Observability-Perf] TPC-C harness scaffold
359. [ ] [Observability-Perf] YCSB harness scaffold
360. [ ] [Observability-Perf] Chaos partition test
361. [ ] [Observability-Perf] Chaos kill-leader test
362. [ ] [Observability-Perf] MVCC GC soak test
363. [ ] [Observability-Perf] Tokio worker thread config
364. [ ] [Observability-Perf] Allocation profiling on KV hot path
365. [ ] [Observability-Perf] Client-side CLI latency stats
366. [ ] [Observability-Perf] Proto compatibility tests
367. [ ] [Observability-Perf] gRPC reflection docs in CI
368. [ ] [Observability-Perf] Feature-flag matrix in getting-started
369. [ ] [Observability-Perf] RUST_LOG examples in docs
370. [ ] [parser] Add unit tests for SQL parser (item 1)
371. [ ] [parser] Add integration tests for SQL parser (item 2)
372. [ ] [parser] Document public API of SQL parser (item 3)
373. [ ] [parser] Reduce allocations in SQL parser (item 4)
374. [ ] [parser] Add tracing spans around SQL parser (item 5)
375. [ ] [parser] Add metrics for SQL parser (item 6)
376. [ ] [parser] Handle error paths in SQL parser (item 7)
377. [ ] [parser] Improve logging in SQL parser (item 8)
378. [ ] [parser] Refactor for clarity in SQL parser (item 9)
379. [ ] [parser] Extract helpers from SQL parser (item 10)
380. [ ] [parser] Benchmark hot path in SQL parser (item 11)
381. [ ] [parser] Fuzz inputs to SQL parser (item 12)
382. [ ] [parser] Add property tests for SQL parser (item 13)
383. [ ] [parser] Validate invariants in SQL parser (item 14)
384. [ ] [parser] Add timeout handling to SQL parser (item 15)
385. [ ] [parser] Add cancellation support to SQL parser (item 16)
386. [ ] [parser] Improve error messages from SQL parser (item 17)
387. [ ] [parser] Add examples using SQL parser (item 18)
388. [ ] [parser] Gate experimental APIs in SQL parser (item 19)
389. [ ] [parser] Stabilize wire format for SQL parser (item 20)
390. [ ] [parser] Add unit tests for SQL parser (item 21)
391. [ ] [parser] Add integration tests for SQL parser (item 22)
392. [ ] [parser] Document public API of SQL parser (item 23)
393. [ ] [parser] Reduce allocations in SQL parser (item 24)
394. [ ] [parser] Add tracing spans around SQL parser (item 25)
395. [ ] [parser] Add metrics for SQL parser (item 26)
396. [ ] [parser] Handle error paths in SQL parser (item 27)
397. [ ] [parser] Improve logging in SQL parser (item 28)
398. [ ] [parser] Refactor for clarity in SQL parser (item 29)
399. [ ] [parser] Extract helpers from SQL parser (item 30)
400. [ ] [parser] Benchmark hot path in SQL parser (item 31)
401. [ ] [parser] Fuzz inputs to SQL parser (item 32)
402. [ ] [parser] Add property tests for SQL parser (item 33)
403. [ ] [parser] Validate invariants in SQL parser (item 34)
404. [ ] [parser] Add timeout handling to SQL parser (item 35)
405. [ ] [parser] Add cancellation support to SQL parser (item 36)
406. [ ] [parser] Improve error messages from SQL parser (item 37)
407. [ ] [parser] Add examples using SQL parser (item 38)
408. [ ] [parser] Gate experimental APIs in SQL parser (item 39)
409. [ ] [parser] Stabilize wire format for SQL parser (item 40)
410. [ ] [planner] Add unit tests for Query planner (item 1)
411. [ ] [planner] Add integration tests for Query planner (item 2)
412. [ ] [planner] Document public API of Query planner (item 3)
413. [ ] [planner] Reduce allocations in Query planner (item 4)
414. [ ] [planner] Add tracing spans around Query planner (item 5)
415. [ ] [planner] Add metrics for Query planner (item 6)
416. [ ] [planner] Handle error paths in Query planner (item 7)
417. [ ] [planner] Improve logging in Query planner (item 8)
418. [ ] [planner] Refactor for clarity in Query planner (item 9)
419. [ ] [planner] Extract helpers from Query planner (item 10)
420. [ ] [planner] Benchmark hot path in Query planner (item 11)
421. [ ] [planner] Fuzz inputs to Query planner (item 12)
422. [ ] [planner] Add property tests for Query planner (item 13)
423. [ ] [planner] Validate invariants in Query planner (item 14)
424. [ ] [planner] Add timeout handling to Query planner (item 15)
425. [ ] [planner] Add cancellation support to Query planner (item 16)
426. [ ] [planner] Improve error messages from Query planner (item 17)
427. [ ] [planner] Add examples using Query planner (item 18)
428. [ ] [planner] Gate experimental APIs in Query planner (item 19)
429. [ ] [planner] Stabilize wire format for Query planner (item 20)
430. [ ] [planner] Add unit tests for Query planner (item 21)
431. [ ] [planner] Add integration tests for Query planner (item 22)
432. [ ] [planner] Document public API of Query planner (item 23)
433. [ ] [planner] Reduce allocations in Query planner (item 24)
434. [ ] [planner] Add tracing spans around Query planner (item 25)
435. [ ] [executor] Add metrics for Query executor (item 1)
436. [ ] [executor] Handle error paths in Query executor (item 2)
437. [ ] [executor] Improve logging in Query executor (item 3)
438. [ ] [executor] Refactor for clarity in Query executor (item 4)
439. [ ] [executor] Extract helpers from Query executor (item 5)
440. [ ] [executor] Benchmark hot path in Query executor (item 6)
441. [ ] [executor] Fuzz inputs to Query executor (item 7)
442. [ ] [executor] Add property tests for Query executor (item 8)
443. [ ] [executor] Validate invariants in Query executor (item 9)
444. [ ] [executor] Add timeout handling to Query executor (item 10)
445. [ ] [executor] Add cancellation support to Query executor (item 11)
446. [ ] [executor] Improve error messages from Query executor (item 12)
447. [ ] [executor] Add examples using Query executor (item 13)
448. [ ] [executor] Gate experimental APIs in Query executor (item 14)
449. [ ] [executor] Stabilize wire format for Query executor (item 15)
450. [ ] [executor] Add unit tests for Query executor (item 16)
451. [ ] [executor] Add integration tests for Query executor (item 17)
452. [ ] [executor] Document public API of Query executor (item 18)
453. [ ] [executor] Reduce allocations in Query executor (item 19)
454. [ ] [executor] Add tracing spans around Query executor (item 20)
455. [ ] [executor] Add metrics for Query executor (item 21)
456. [ ] [executor] Handle error paths in Query executor (item 22)
457. [ ] [executor] Improve logging in Query executor (item 23)
458. [ ] [executor] Refactor for clarity in Query executor (item 24)
459. [ ] [executor] Extract helpers from Query executor (item 25)
460. [ ] [executor] Benchmark hot path in Query executor (item 26)
461. [ ] [executor] Fuzz inputs to Query executor (item 27)
462. [ ] [executor] Add property tests for Query executor (item 28)
463. [ ] [executor] Validate invariants in Query executor (item 29)
464. [ ] [executor] Add timeout handling to Query executor (item 30)
465. [ ] [executor] Add cancellation support to Query executor (item 31)
466. [ ] [executor] Improve error messages from Query executor (item 32)
467. [ ] [executor] Add examples using Query executor (item 33)
468. [ ] [executor] Gate experimental APIs in Query executor (item 34)
469. [ ] [executor] Stabilize wire format for Query executor (item 35)
470. [ ] [aggregation] Add unit tests for Aggregation (item 1)
471. [ ] [aggregation] Add integration tests for Aggregation (item 2)
472. [ ] [aggregation] Document public API of Aggregation (item 3)
473. [ ] [aggregation] Reduce allocations in Aggregation (item 4)
474. [ ] [aggregation] Add tracing spans around Aggregation (item 5)
475. [ ] [aggregation] Add metrics for Aggregation (item 6)
476. [ ] [aggregation] Handle error paths in Aggregation (item 7)
477. [ ] [aggregation] Improve logging in Aggregation (item 8)
478. [ ] [aggregation] Refactor for clarity in Aggregation (item 9)
479. [ ] [aggregation] Extract helpers from Aggregation (item 10)
480. [ ] [aggregation] Benchmark hot path in Aggregation (item 11)
481. [ ] [aggregation] Fuzz inputs to Aggregation (item 12)
482. [ ] [aggregation] Add property tests for Aggregation (item 13)
483. [ ] [aggregation] Validate invariants in Aggregation (item 14)
484. [ ] [aggregation] Add timeout handling to Aggregation (item 15)
485. [ ] [cache] Add cancellation support to Query cache (item 1)
486. [ ] [cache] Improve error messages from Query cache (item 2)
487. [ ] [cache] Add examples using Query cache (item 3)
488. [ ] [cache] Gate experimental APIs in Query cache (item 4)
489. [ ] [cache] Stabilize wire format for Query cache (item 5)
490. [ ] [cache] Add unit tests for Query cache (item 6)
491. [ ] [cache] Add integration tests for Query cache (item 7)
492. [ ] [cache] Document public API of Query cache (item 8)
493. [ ] [cache] Reduce allocations in Query cache (item 9)
494. [ ] [cache] Add tracing spans around Query cache (item 10)
495. [ ] [cache] Add metrics for Query cache (item 11)
496. [ ] [cache] Handle error paths in Query cache (item 12)
497. [ ] [prepared] Improve logging in Prepared statements (item 1)
498. [ ] [prepared] Refactor for clarity in Prepared statements (item 2)
499. [ ] [prepared] Extract helpers from Prepared statements (item 3)
500. [ ] [prepared] Benchmark hot path in Prepared statements (item 4)
501. [ ] [prepared] Fuzz inputs to Prepared statements (item 5)
502. [ ] [prepared] Add property tests for Prepared statements (item 6)
503. [ ] [prepared] Validate invariants in Prepared statements (item 7)
504. [ ] [prepared] Add timeout handling to Prepared statements (item 8)
505. [ ] [prepared] Add cancellation support to Prepared statements (item 9)
506. [ ] [prepared] Improve error messages from Prepared statements (item 10)
507. [ ] [prepared] Add examples using Prepared statements (item 11)
508. [ ] [prepared] Gate experimental APIs in Prepared statements (item 12)
509. [ ] [statistics] Stabilize wire format for Table statistics (item 1)
510. [ ] [statistics] Add unit tests for Table statistics (item 2)
511. [ ] [statistics] Add integration tests for Table statistics (item 3)
512. [ ] [statistics] Document public API of Table statistics (item 4)
513. [ ] [statistics] Reduce allocations in Table statistics (item 5)
514. [ ] [statistics] Add tracing spans around Table statistics (item 6)
515. [ ] [statistics] Add metrics for Table statistics (item 7)
516. [ ] [statistics] Handle error paths in Table statistics (item 8)
517. [ ] [statistics] Improve logging in Table statistics (item 9)
518. [ ] [statistics] Refactor for clarity in Table statistics (item 10)
519. [ ] [statistics] Extract helpers from Table statistics (item 11)
520. [ ] [statistics] Benchmark hot path in Table statistics (item 12)
521. [ ] [raft-log] Fuzz inputs to Raft log (item 1)
522. [ ] [raft-log] Add property tests for Raft log (item 2)
523. [ ] [raft-log] Validate invariants in Raft log (item 3)
524. [ ] [raft-log] Add timeout handling to Raft log (item 4)
525. [ ] [raft-log] Add cancellation support to Raft log (item 5)
526. [ ] [raft-log] Improve error messages from Raft log (item 6)
527. [ ] [raft-log] Add examples using Raft log (item 7)
528. [ ] [raft-log] Gate experimental APIs in Raft log (item 8)
529. [ ] [raft-log] Stabilize wire format for Raft log (item 9)
530. [ ] [raft-log] Add unit tests for Raft log (item 10)
531. [ ] [raft-log] Add integration tests for Raft log (item 11)
532. [ ] [raft-log] Document public API of Raft log (item 12)
533. [ ] [raft-log] Reduce allocations in Raft log (item 13)
534. [ ] [raft-log] Add tracing spans around Raft log (item 14)
535. [ ] [raft-log] Add metrics for Raft log (item 15)
536. [ ] [raft-log] Handle error paths in Raft log (item 16)
537. [ ] [raft-log] Improve logging in Raft log (item 17)
538. [ ] [raft-log] Refactor for clarity in Raft log (item 18)
539. [ ] [raft-log] Extract helpers from Raft log (item 19)
540. [ ] [raft-log] Benchmark hot path in Raft log (item 20)
541. [ ] [raft-state] Fuzz inputs to Raft volatile state (item 1)
542. [ ] [raft-state] Add property tests for Raft volatile state (item 2)
543. [ ] [raft-state] Validate invariants in Raft volatile state (item 3)
544. [ ] [raft-state] Add timeout handling to Raft volatile state (item 4)
545. [ ] [raft-state] Add cancellation support to Raft volatile state (item 5)
546. [ ] [raft-state] Improve error messages from Raft volatile state (item 6)
547. [ ] [raft-state] Add examples using Raft volatile state (item 7)
548. [ ] [raft-state] Gate experimental APIs in Raft volatile state (item 8)
549. [ ] [raft-state] Stabilize wire format for Raft volatile state (item 9)
550. [ ] [raft-state] Add unit tests for Raft volatile state (item 10)
551. [ ] [raft-state] Add integration tests for Raft volatile state (item 11)
552. [ ] [raft-state] Document public API of Raft volatile state (item 12)
553. [ ] [raft-state] Reduce allocations in Raft volatile state (item 13)
554. [ ] [raft-state] Add tracing spans around Raft volatile state (item 14)
555. [ ] [raft-state] Add metrics for Raft volatile state (item 15)
556. [ ] [raft-rpc] Handle error paths in Raft RPC codec (item 1)
557. [ ] [raft-rpc] Improve logging in Raft RPC codec (item 2)
558. [ ] [raft-rpc] Refactor for clarity in Raft RPC codec (item 3)
559. [ ] [raft-rpc] Extract helpers from Raft RPC codec (item 4)
560. [ ] [raft-rpc] Benchmark hot path in Raft RPC codec (item 5)
561. [ ] [raft-rpc] Fuzz inputs to Raft RPC codec (item 6)
562. [ ] [raft-rpc] Add property tests for Raft RPC codec (item 7)
563. [ ] [raft-rpc] Validate invariants in Raft RPC codec (item 8)
564. [ ] [raft-rpc] Add timeout handling to Raft RPC codec (item 9)
565. [ ] [raft-rpc] Add cancellation support to Raft RPC codec (item 10)
566. [ ] [raft-rpc] Improve error messages from Raft RPC codec (item 11)
567. [ ] [raft-rpc] Add examples using Raft RPC codec (item 12)
568. [ ] [raft-rpc] Gate experimental APIs in Raft RPC codec (item 13)
569. [ ] [raft-rpc] Stabilize wire format for Raft RPC codec (item 14)
570. [ ] [raft-rpc] Add unit tests for Raft RPC codec (item 15)
571. [ ] [raft-snapshot] Add integration tests for Raft snapshot (item 1)
572. [ ] [raft-snapshot] Document public API of Raft snapshot (item 2)
573. [ ] [raft-snapshot] Reduce allocations in Raft snapshot (item 3)
574. [ ] [raft-snapshot] Add tracing spans around Raft snapshot (item 4)
575. [ ] [raft-snapshot] Add metrics for Raft snapshot (item 5)
576. [ ] [raft-snapshot] Handle error paths in Raft snapshot (item 6)
577. [ ] [raft-snapshot] Improve logging in Raft snapshot (item 7)
578. [ ] [raft-snapshot] Refactor for clarity in Raft snapshot (item 8)
579. [ ] [raft-snapshot] Extract helpers from Raft snapshot (item 9)
580. [ ] [raft-snapshot] Benchmark hot path in Raft snapshot (item 10)
581. [ ] [raft-snapshot] Fuzz inputs to Raft snapshot (item 11)
582. [ ] [raft-snapshot] Add property tests for Raft snapshot (item 12)
583. [ ] [raft-snapshot] Validate invariants in Raft snapshot (item 13)
584. [ ] [raft-snapshot] Add timeout handling to Raft snapshot (item 14)
585. [ ] [raft-snapshot] Add cancellation support to Raft snapshot (item 15)
586. [ ] [tx-manager] Improve error messages from Transaction manager (item 1)
587. [ ] [tx-manager] Add examples using Transaction manager (item 2)
588. [ ] [tx-manager] Gate experimental APIs in Transaction manager (item 3)
589. [ ] [tx-manager] Stabilize wire format for Transaction manager (item 4)
590. [ ] [tx-manager] Add unit tests for Transaction manager (item 5)
591. [ ] [tx-manager] Add integration tests for Transaction manager (item 6)
592. [ ] [tx-manager] Document public API of Transaction manager (item 7)
593. [ ] [tx-manager] Reduce allocations in Transaction manager (item 8)
594. [ ] [tx-manager] Add tracing spans around Transaction manager (item 9)
595. [ ] [tx-manager] Add metrics for Transaction manager (item 10)
596. [ ] [tx-manager] Handle error paths in Transaction manager (item 11)
597. [ ] [tx-manager] Improve logging in Transaction manager (item 12)
598. [ ] [tx-manager] Refactor for clarity in Transaction manager (item 13)
599. [ ] [tx-manager] Extract helpers from Transaction manager (item 14)
600. [ ] [tx-manager] Benchmark hot path in Transaction manager (item 15)
601. [ ] [tx-manager] Fuzz inputs to Transaction manager (item 16)
602. [ ] [tx-manager] Add property tests for Transaction manager (item 17)
603. [ ] [tx-manager] Validate invariants in Transaction manager (item 18)
604. [ ] [tx-manager] Add timeout handling to Transaction manager (item 19)
605. [ ] [tx-manager] Add cancellation support to Transaction manager (item 20)
606. [ ] [tx-mvcc] Improve error messages from Transaction MVCC detector (item 1)
607. [ ] [tx-mvcc] Add examples using Transaction MVCC detector (item 2)
608. [ ] [tx-mvcc] Gate experimental APIs in Transaction MVCC detector (item 3)
609. [ ] [tx-mvcc] Stabilize wire format for Transaction MVCC detector (item 4)
610. [ ] [tx-mvcc] Add unit tests for Transaction MVCC detector (item 5)
611. [ ] [tx-mvcc] Add integration tests for Transaction MVCC detector (item 6)
612. [ ] [tx-mvcc] Document public API of Transaction MVCC detector (item 7)
613. [ ] [tx-mvcc] Reduce allocations in Transaction MVCC detector (item 8)
614. [ ] [tx-mvcc] Add tracing spans around Transaction MVCC detector (item 9)
615. [ ] [tx-mvcc] Add metrics for Transaction MVCC detector (item 10)
616. [ ] [tx-mvcc] Handle error paths in Transaction MVCC detector (item 11)
617. [ ] [tx-mvcc] Improve logging in Transaction MVCC detector (item 12)
618. [ ] [tx-mvcc] Refactor for clarity in Transaction MVCC detector (item 13)
619. [ ] [tx-mvcc] Extract helpers from Transaction MVCC detector (item 14)
620. [ ] [tx-mvcc] Benchmark hot path in Transaction MVCC detector (item 15)
621. [ ] [network-conn] Fuzz inputs to Connection pool (item 1)
622. [ ] [network-conn] Add property tests for Connection pool (item 2)
623. [ ] [network-conn] Validate invariants in Connection pool (item 3)
624. [ ] [network-conn] Add timeout handling to Connection pool (item 4)
625. [ ] [network-conn] Add cancellation support to Connection pool (item 5)
626. [ ] [network-conn] Improve error messages from Connection pool (item 6)
627. [ ] [network-conn] Add examples using Connection pool (item 7)
628. [ ] [network-conn] Gate experimental APIs in Connection pool (item 8)
629. [ ] [network-conn] Stabilize wire format for Connection pool (item 9)
630. [ ] [network-conn] Add unit tests for Connection pool (item 10)
631. [ ] [network-conn] Add integration tests for Connection pool (item 11)
632. [ ] [network-conn] Document public API of Connection pool (item 12)
633. [ ] [network-conn] Reduce allocations in Connection pool (item 13)
634. [ ] [network-conn] Add tracing spans around Connection pool (item 14)
635. [ ] [network-conn] Add metrics for Connection pool (item 15)
636. [ ] [network-client] Handle error paths in Network client (item 1)
637. [ ] [network-client] Improve logging in Network client (item 2)
638. [ ] [network-client] Refactor for clarity in Network client (item 3)
639. [ ] [network-client] Extract helpers from Network client (item 4)
640. [ ] [network-client] Benchmark hot path in Network client (item 5)
641. [ ] [network-client] Fuzz inputs to Network client (item 6)
642. [ ] [network-client] Add property tests for Network client (item 7)
643. [ ] [network-client] Validate invariants in Network client (item 8)
644. [ ] [network-client] Add timeout handling to Network client (item 9)
645. [ ] [network-client] Add cancellation support to Network client (item 10)
646. [ ] [network-client] Improve error messages from Network client (item 11)
647. [ ] [network-client] Add examples using Network client (item 12)
648. [ ] [config-validator] Gate experimental APIs in Config validator (item 1)
649. [ ] [config-validator] Stabilize wire format for Config validator (item 2)
650. [ ] [config-validator] Add unit tests for Config validator (item 3)
651. [ ] [config-validator] Add integration tests for Config validator (item 4)
652. [ ] [config-validator] Document public API of Config validator (item 5)
653. [ ] [config-validator] Reduce allocations in Config validator (item 6)
654. [ ] [config-validator] Add tracing spans around Config validator (item 7)
655. [ ] [config-validator] Add metrics for Config validator (item 8)
656. [ ] [config-validator] Handle error paths in Config validator (item 9)
657. [ ] [config-validator] Improve logging in Config validator (item 10)
658. [ ] [config-validator] Refactor for clarity in Config validator (item 11)
659. [ ] [config-validator] Extract helpers from Config validator (item 12)
660. [ ] [config-validator] Benchmark hot path in Config validator (item 13)
661. [ ] [config-validator] Fuzz inputs to Config validator (item 14)
662. [ ] [config-validator] Add property tests for Config validator (item 15)
663. [ ] [cli-repl] Validate invariants in CLI REPL (item 1)
664. [ ] [cli-repl] Add timeout handling to CLI REPL (item 2)
665. [ ] [cli-repl] Add cancellation support to CLI REPL (item 3)
666. [ ] [cli-repl] Improve error messages from CLI REPL (item 4)
667. [ ] [cli-repl] Add examples using CLI REPL (item 5)
668. [ ] [cli-repl] Gate experimental APIs in CLI REPL (item 6)
669. [ ] [cli-repl] Stabilize wire format for CLI REPL (item 7)
670. [ ] [cli-repl] Add unit tests for CLI REPL (item 8)
671. [ ] [cli-repl] Add integration tests for CLI REPL (item 9)
672. [ ] [cli-repl] Document public API of CLI REPL (item 10)
673. [ ] [cli-repl] Reduce allocations in CLI REPL (item 11)
674. [ ] [cli-repl] Add tracing spans around CLI REPL (item 12)
675. [ ] [cli-import] Add metrics for CLI import/export (item 1)
676. [ ] [cli-import] Handle error paths in CLI import/export (item 2)
677. [ ] [cli-import] Improve logging in CLI import/export (item 3)
678. [ ] [cli-import] Refactor for clarity in CLI import/export (item 4)
679. [ ] [cli-import] Extract helpers from CLI import/export (item 5)
680. [ ] [cli-import] Benchmark hot path in CLI import/export (item 6)
681. [ ] [cli-import] Fuzz inputs to CLI import/export (item 7)
682. [ ] [cli-import] Add property tests for CLI import/export (item 8)
683. [ ] [cli-import] Validate invariants in CLI import/export (item 9)
684. [ ] [cli-import] Add timeout handling to CLI import/export (item 10)
685. [ ] [cli-import] Add cancellation support to CLI import/export (item 11)
686. [ ] [cli-import] Improve error messages from CLI import/export (item 12)
687. [ ] [memory-engine] Add examples using Memory engine (item 1)
688. [ ] [memory-engine] Gate experimental APIs in Memory engine (item 2)
689. [ ] [memory-engine] Stabilize wire format for Memory engine (item 3)
690. [ ] [memory-engine] Add unit tests for Memory engine (item 4)
691. [ ] [memory-engine] Add integration tests for Memory engine (item 5)
692. [ ] [memory-engine] Document public API of Memory engine (item 6)
693. [ ] [memory-engine] Reduce allocations in Memory engine (item 7)
694. [ ] [memory-engine] Add tracing spans around Memory engine (item 8)
695. [ ] [memory-engine] Add metrics for Memory engine (item 9)
696. [ ] [memory-engine] Handle error paths in Memory engine (item 10)
697. [ ] [sled-engine] Improve logging in Sled engine (item 1)
698. [ ] [sled-engine] Refactor for clarity in Sled engine (item 2)
699. [ ] [sled-engine] Extract helpers from Sled engine (item 3)
700. [ ] [sled-engine] Benchmark hot path in Sled engine (item 4)
701. [ ] [sled-engine] Fuzz inputs to Sled engine (item 5)
702. [ ] [sled-engine] Add property tests for Sled engine (item 6)
703. [ ] [sled-engine] Validate invariants in Sled engine (item 7)
704. [ ] [sled-engine] Add timeout handling to Sled engine (item 8)
705. [ ] [sled-engine] Add cancellation support to Sled engine (item 9)
706. [ ] [sled-engine] Improve error messages from Sled engine (item 10)
707. [ ] [sled-engine] Add examples using Sled engine (item 11)
708. [ ] [sled-engine] Gate experimental APIs in Sled engine (item 12)
709. [ ] [rocks-engine] Stabilize wire format for RocksDB engine (item 1)
710. [ ] [rocks-engine] Add unit tests for RocksDB engine (item 2)
711. [ ] [rocks-engine] Add integration tests for RocksDB engine (item 3)
712. [ ] [rocks-engine] Document public API of RocksDB engine (item 4)
713. [ ] [rocks-engine] Reduce allocations in RocksDB engine (item 5)
714. [ ] [rocks-engine] Add tracing spans around RocksDB engine (item 6)
715. [ ] [rocks-engine] Add metrics for RocksDB engine (item 7)
716. [ ] [rocks-engine] Handle error paths in RocksDB engine (item 8)
717. [ ] [rocks-engine] Improve logging in RocksDB engine (item 9)
718. [ ] [rocks-engine] Refactor for clarity in RocksDB engine (item 10)
719. [ ] [rocks-engine] Extract helpers from RocksDB engine (item 11)
720. [ ] [rocks-engine] Benchmark hot path in RocksDB engine (item 12)
721. [ ] [index-mgr] Fuzz inputs to Index manager (item 1)
722. [ ] [index-mgr] Add property tests for Index manager (item 2)
723. [ ] [index-mgr] Validate invariants in Index manager (item 3)
724. [ ] [index-mgr] Add timeout handling to Index manager (item 4)
725. [ ] [index-mgr] Add cancellation support to Index manager (item 5)
726. [ ] [index-mgr] Improve error messages from Index manager (item 6)
727. [ ] [index-mgr] Add examples using Index manager (item 7)
728. [ ] [index-mgr] Gate experimental APIs in Index manager (item 8)
729. [ ] [index-mgr] Stabilize wire format for Index manager (item 9)
730. [ ] [index-mgr] Add unit tests for Index manager (item 10)
731. [ ] [index-mgr] Add integration tests for Index manager (item 11)
732. [ ] [index-mgr] Document public API of Index manager (item 12)
733. [ ] [iterator] Reduce allocations in Storage iterator (item 1)
734. [ ] [iterator] Add tracing spans around Storage iterator (item 2)
735. [ ] [iterator] Add metrics for Storage iterator (item 3)
736. [ ] [iterator] Handle error paths in Storage iterator (item 4)
737. [ ] [iterator] Improve logging in Storage iterator (item 5)
738. [ ] [iterator] Refactor for clarity in Storage iterator (item 6)
739. [ ] [iterator] Extract helpers from Storage iterator (item 7)
740. [ ] [iterator] Benchmark hot path in Storage iterator (item 8)
741. [ ] [iterator] Fuzz inputs to Storage iterator (item 9)
742. [ ] [iterator] Add property tests for Storage iterator (item 10)
743. [ ] [proto] Validate invariants in Proto API (item 1)
744. [ ] [proto] Add timeout handling to Proto API (item 2)
745. [ ] [proto] Add cancellation support to Proto API (item 3)
746. [ ] [proto] Improve error messages from Proto API (item 4)
747. [ ] [proto] Add examples using Proto API (item 5)
748. [ ] [proto] Gate experimental APIs in Proto API (item 6)
749. [ ] [proto] Stabilize wire format for Proto API (item 7)
750. [ ] [proto] Add unit tests for Proto API (item 8)
751. [ ] [proto] Add integration tests for Proto API (item 9)
752. [ ] [proto] Document public API of Proto API (item 10)
753. [ ] [proto] Reduce allocations in Proto API (item 11)
754. [ ] [proto] Add tracing spans around Proto API (item 12)
755. [ ] [proto] Add metrics for Proto API (item 13)
756. [ ] [proto] Handle error paths in Proto API (item 14)
757. [ ] [proto] Improve logging in Proto API (item 15)
758. [ ] [proto] Refactor for clarity in Proto API (item 16)
759. [ ] [proto] Extract helpers from Proto API (item 17)
760. [ ] [proto] Benchmark hot path in Proto API (item 18)
761. [ ] [proto] Fuzz inputs to Proto API (item 19)
762. [ ] [proto] Add property tests for Proto API (item 20)
763. [ ] [docker] Validate invariants in Docker/Compose (item 1)
764. [ ] [docker] Add timeout handling to Docker/Compose (item 2)
765. [ ] [docker] Add cancellation support to Docker/Compose (item 3)
766. [ ] [docker] Improve error messages from Docker/Compose (item 4)
767. [ ] [docker] Add examples using Docker/Compose (item 5)
768. [ ] [docker] Gate experimental APIs in Docker/Compose (item 6)
769. [ ] [docker] Stabilize wire format for Docker/Compose (item 7)
770. [ ] [docker] Add unit tests for Docker/Compose (item 8)
771. [ ] [docker] Add integration tests for Docker/Compose (item 9)
772. [ ] [docker] Document public API of Docker/Compose (item 10)
773. [ ] [docker] Reduce allocations in Docker/Compose (item 11)
774. [ ] [docker] Add tracing spans around Docker/Compose (item 12)
775. [ ] [docker] Add metrics for Docker/Compose (item 13)
776. [ ] [docker] Handle error paths in Docker/Compose (item 14)
777. [ ] [docker] Improve logging in Docker/Compose (item 15)
778. [ ] [ci] Refactor for clarity in CI workflow (item 1)
779. [ ] [ci] Extract helpers from CI workflow (item 2)
780. [ ] [ci] Benchmark hot path in CI workflow (item 3)
781. [ ] [ci] Fuzz inputs to CI workflow (item 4)
782. [ ] [ci] Add property tests for CI workflow (item 5)
783. [ ] [ci] Validate invariants in CI workflow (item 6)
784. [ ] [ci] Add timeout handling to CI workflow (item 7)
785. [ ] [ci] Add cancellation support to CI workflow (item 8)
786. [ ] [ci] Improve error messages from CI workflow (item 9)
787. [ ] [ci] Add examples using CI workflow (item 10)
788. [ ] [ci] Gate experimental APIs in CI workflow (item 11)
789. [ ] [ci] Stabilize wire format for CI workflow (item 12)
790. [ ] [ci] Add unit tests for CI workflow (item 13)
791. [ ] [ci] Add integration tests for CI workflow (item 14)
792. [ ] [ci] Document public API of CI workflow (item 15)
793. [ ] [ci] Reduce allocations in CI workflow (item 16)
794. [ ] [ci] Add tracing spans around CI workflow (item 17)
795. [ ] [ci] Add metrics for CI workflow (item 18)
796. [ ] [ci] Handle error paths in CI workflow (item 19)
797. [ ] [ci] Improve logging in CI workflow (item 20)
798. [ ] [docs] Refactor for clarity in Documentation accuracy (item 1)
799. [ ] [docs] Extract helpers from Documentation accuracy (item 2)
800. [ ] [docs] Benchmark hot path in Documentation accuracy (item 3)
801. [ ] [docs] Fuzz inputs to Documentation accuracy (item 4)
802. [ ] [docs] Add property tests for Documentation accuracy (item 5)
803. [ ] [docs] Validate invariants in Documentation accuracy (item 6)
804. [ ] [docs] Add timeout handling to Documentation accuracy (item 7)
805. [ ] [docs] Add cancellation support to Documentation accuracy (item 8)
806. [ ] [docs] Improve error messages from Documentation accuracy (item 9)
807. [ ] [docs] Add examples using Documentation accuracy (item 10)
808. [ ] [docs] Gate experimental APIs in Documentation accuracy (item 11)
809. [ ] [docs] Stabilize wire format for Documentation accuracy (item 12)
810. [ ] [docs] Add unit tests for Documentation accuracy (item 13)
811. [ ] [docs] Add integration tests for Documentation accuracy (item 14)
812. [ ] [docs] Document public API of Documentation accuracy (item 15)
813. [ ] [bench] Reduce allocations in Benchmarking (item 1)
814. [ ] [bench] Add tracing spans around Benchmarking (item 2)
815. [ ] [bench] Add metrics for Benchmarking (item 3)
816. [ ] [bench] Handle error paths in Benchmarking (item 4)
817. [ ] [bench] Improve logging in Benchmarking (item 5)
818. [ ] [bench] Refactor for clarity in Benchmarking (item 6)
819. [ ] [bench] Extract helpers from Benchmarking (item 7)
820. [ ] [bench] Benchmark hot path in Benchmarking (item 8)
821. [ ] [bench] Fuzz inputs to Benchmarking (item 9)
822. [ ] [bench] Add property tests for Benchmarking (item 10)
823. [ ] [security] Validate invariants in Security hardening (item 1)
824. [ ] [security] Add timeout handling to Security hardening (item 2)
825. [ ] [security] Add cancellation support to Security hardening (item 3)
826. [ ] [security] Improve error messages from Security hardening (item 4)
827. [ ] [security] Add examples using Security hardening (item 5)
828. [ ] [security] Gate experimental APIs in Security hardening (item 6)
829. [ ] [security] Stabilize wire format for Security hardening (item 7)
830. [ ] [security] Add unit tests for Security hardening (item 8)
831. [ ] [security] Add integration tests for Security hardening (item 9)
832. [ ] [security] Document public API of Security hardening (item 10)
833. [ ] [security] Reduce allocations in Security hardening (item 11)
834. [ ] [security] Add tracing spans around Security hardening (item 12)
835. [ ] [security] Add metrics for Security hardening (item 13)
836. [ ] [security] Handle error paths in Security hardening (item 14)
837. [ ] [security] Improve logging in Security hardening (item 15)
838. [ ] [ops] Refactor for clarity in Operations/tooling (item 1)
839. [ ] [ops] Extract helpers from Operations/tooling (item 2)
840. [ ] [ops] Benchmark hot path in Operations/tooling (item 3)
841. [ ] [ops] Fuzz inputs to Operations/tooling (item 4)
842. [ ] [ops] Add property tests for Operations/tooling (item 5)
843. [ ] [ops] Validate invariants in Operations/tooling (item 6)
844. [ ] [ops] Add timeout handling to Operations/tooling (item 7)
845. [ ] [ops] Add cancellation support to Operations/tooling (item 8)
846. [ ] [ops] Improve error messages from Operations/tooling (item 9)
847. [ ] [ops] Add examples using Operations/tooling (item 10)
848. [ ] [ops] Gate experimental APIs in Operations/tooling (item 11)
849. [ ] [ops] Stabilize wire format for Operations/tooling (item 12)
850. [ ] [ops] Add unit tests for Operations/tooling (item 13)
851. [ ] [ops] Add integration tests for Operations/tooling (item 14)
852. [ ] [ops] Document public API of Operations/tooling (item 15)

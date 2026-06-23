# 🏦 Stablecoin Payment System

> A high-performance, dual-implementation (Anchor reference + native zero-copy) Solana stablecoin payment rail with an enterprise-grade orchestrator, idempotent APIs, webhook reliability, and future-proof on-chain state.

## 📑 Table of Contents
1. [System Architecture](#-system-architecture)
2. [Directory Structure](#-directory-structure)
3. [Master Invariant List](#-master-invariant-list)
4. [Subsystem Threat Matrix & Mitigations](#-subsystem-threat-matrix--mitigations)

---

## 🏛️ System Architecture

### 1. Macro Execution Flow (Web-First NFC to Settlement)
The logical data flow enforces strict separation of concerns. The Web PWA handles the physical tap simulation and cryptographic signing, while the Orchestrator acts solely as a non-custodial audit and routing layer, bypassing mempool latency via direct TPU/Jito submission.

```mermaid
flowchart TD
    subgraph Client ["Client Layer (TypeScript SDK)"]
        A[Web PWA] -->|NFC Tap Simulation| B(Construct Raw TX)
        B -->|Sign & Pack| C[User Signature Payload]
    end

    subgraph Backend ["Orchestrator (Rust)"]
        C -->|POST /api/v1/transfer| D{Audit & Validation Layer}
        D -->|Verify Nonce/Sig| E[(PostgreSQL: Idempotency)]
        E -->|State: Pending| F[Jito Bundle / TPU Client]
        F -->|MEV Protected Submission| G((Solana Network))
    end

    subgraph OnChain ["Solana Runtime (BPF)"]
        G --> H{Instruction Router}
        H -->|Anchor Crate| I[Borsh Deserialization]
        H -->|Native Crate| J[Zero-Copy Bytemuck Cast]
        J --> K[(User Vault State)]
    end

    subgraph Settlement ["Exit & Notification"]
        K -.->|Finalized| L[Transaction Poller]
        L --> M[Update DB: Confirmed]
        M --> N[Webhook Dispatcher]
        N --> O[Merchant Server]
    end

```

### 2. Idempotency & Delivery State Machine

Network partitions will happen. The system guarantees exact-once execution using immutable database constraints mapped to Solana transaction signatures.

```mermaid
stateDiagram-v2
    [*] --> Pending : Receive Valid Signature
    Pending --> OnChainSubmission : TPU / Jito Route
    
    OnChainSubmission --> Pending : Timeout / Blockhash Expired
    OnChainSubmission --> Confirmed : Solana Finality Reached
    OnChainSubmission --> Failed : Instruction Error (e.g. Slippage)
    
    Confirmed --> WebhookDispatch : Trigger Event
    WebhookDispatch --> Delivered : Merchant 200 OK
    WebhookDispatch --> DeadLetterQueue : Max Retries Exceeded
    
    Failed --> [*]
    Delivered --> [*]
    DeadLetterQueue --> [*]

```

### 3. Native Zero-Copy Memory Layout (#[repr(C)])

To survive the BPF VM alignment rules without reallocation, the `StablecoinState` account must be an immutable byte window.

```mermaid
classDiagram
    class UserVaultState {
        <<Zero-Copy #[repr(C)]>>
        +u8 version (Byte 0)
        +u8 bump (Byte 1)
        +[u8; 6] _alignment_padding0 (Bytes 2-7)
        +Pubkey owner (Bytes 8-39)
        +Pubkey primary_delegate (Bytes 40-71)
        +Pubkey governance_authority (Bytes 72-103)
        +u64 collateral_balance (Bytes 104-111)
        +u64 debt_balance (Bytes 112-119)
        +u64 state_flags (Bytes 120-127)
        +[u8; 256] reserved_buffer (Bytes 128-383)
    }

```

---

## 📂 Directory Structure

```text
stablecoin-payment-system/
├── README.md
├── docker-compose.yml
├── Makefile
├── .env.example
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── deploy.yml
│
├── onchain/                           # Multi-Crate Performance Sandbox
│   ├── Anchor.toml                    # Orchestrates both reference and native targets
│   ├── Cargo.toml                     # Workspace root grouping the independent program crates
│   ├── benches/                       # ⚡ Side-by-side Compute Unit profiling modules
│   │   └── execution_footprint.rs
│   ├── migrations/
│   │   └── deploy.ts
│   ├── programs/
│   │   ├── anchor_stablecoin/         # 🏢 High-Level Framework Reference Crate
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   │       ├── lib.rs             # Injects full Borsh/Anchor routing stack
│   │   │       ├── errors.rs
│   │   │       ├── instructions/
│   │   │       │   ├── initialize.rs
│   │   │       │   ├── mint_tokens.rs
│   │   │       │   └── burn_tokens.rs
│   │   │       └── state/
│   │   │           └── account_layouts.rs
│   │   │
│   │   └── native_stablecoin/         # 🚀 Ultra-Performance Native Zero-Copy Crate
│   │       ├── Cargo.toml
│   │       └── src/
│   │           ├── entrypoint.rs      # Naked entrypoint!(process_instruction); with zero framework bloat
│   │           ├── processor.rs       # Direct C-ABI instruction variant execution router
│   │           ├── errors.rs
│   │           ├── instructions/
│   │           │   ├── initialize.rs  # Uses raw zero-copy byte window modification
│   │           │   ├── mint.rs
│   │           │   ├── burn.rs
│   │           │   └── transfer_hook.rs
│   │           ├── state.rs           # Minimalist memory layouts with unaligned primitive arrays
│   │           └── utils/
│   │               ├── mod.rs
│   │               └── zerocopy_parser.rs # 🧠 Explicit bytemuck pointer-casting engine
│   └── tests/
│       ├── anchor_suite.test.ts
│       ├── native_suite.test.ts
│       └── fuzz/                      # 🛡️ Trident property-based invariant testing suite
│           ├── Cargo.toml
│           ├── fuzz_config.toml
│           └── instructions.rs
│
├── orchestrator/                      # High-Velocity Corporate Routing Server (Rust)
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── build.rs
│   ├── rustfmt.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── config/
│   │   │   ├── mod.rs
│   │   │   ├── settings.rs
│   │   │   └── solana_config.rs
│   │   ├── domain/
│   │   │   ├── mod.rs
│   │   │   ├── payment.rs
│   │   │   ├── idempotency_key.rs
│   │   │   ├── webhook_event.rs
│   │   │   └── error.rs
│   │   ├── application/
│   │   │   ├── mod.rs
│   │   │   ├── mint_service.rs
│   │   │   ├── transfer_service.rs
│   │   │   ├── burn_service.rs
│   │   │   └── webhook_dispatcher.rs
│   │   ├── infrastructure/
│   │   │   ├── mod.rs
│   │   │   ├── kms/                   # 🔑 AWS / GCP Cloud KMS secure signer layer
│   │   │   │   ├── mod.rs
│   │   │   │   └── aws_client.rs
│   │   │   ├── queue/                 # 📦 Kafka event pipeline / Dead Letter Queues
│   │   │   │   ├── mod.rs
│   │   │   │   └── kafka_producer.rs
│   │   │   ├── solana/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── client.rs          # Connection + dynamic priority fee calculations
│   │   │   │   ├── jito_bundle.rs     # 🌪️ Congestion-resilient MEV bundle submission
│   │   │   │   ├── tpu_client.rs      # 🚀 Direct-to-validator transaction ingestion
│   │   │   │   ├── transaction_builder.rs # Compiles raw packed buffers for the Native target
│   │   │   │   └── rpc_ext.rs
│   │   │   ├── db/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── postgres.rs
│   │   │   │   ├── repositories/
│   │   │   │   │   ├── payment_repo.rs
│   │   │   │   │   ├── idempotency_repo.rs
│   │   │   │   │   └── webhook_repo.rs
│   │   │   │   └── migrations/
│   │   │   │       ├── 20260101000000_init.sql
│   │   │   │       └── 20260102000000_add_webhook_delivery.sql
│   │   │   ├── cache/
│   │   │   │   ├── mod.rs
│   │   │   │   └── redis.rs
│   │   │   ├── webhook/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── sender.rs
│   │   │   │   └── signature.rs
│   │   │   └── metrics/
│   │   │       ├── mod.rs
│   │   │       └── prometheus.rs
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── http/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── server.rs
│   │   │   │   ├── middleware/
│   │   │   │   │   ├── auth.rs
│   │   │   │   │   ├── rate_limiter.rs
│   │   │   │   │   ├── request_id.rs
│   │   │   │   │   └── logging.rs
│   │   │   │   ├── dto/
│   │   │   │   │   ├── request.rs
│   │   │   │   │   └── response.rs
│   │   │   │   └── handlers/
│   │   │   │       ├── mint.rs
│   │   │   │       ├── transfer.rs
│   │   │   │       ├── burn.rs
│   │   │   │       ├── webhook.rs
│   │   │   │       └── health.rs
│   │   │   └── websocket/
│   │   │       └── mod.rs
│   │   └── jobs/
│   │       ├── mod.rs
│   │       ├── webhook_retry_worker.rs
│   │       └── transaction_confirmation_poller.rs
│   └── tests/
│       ├── integration/
│       │   ├── mint_api_test.rs
│       │   ├── idempotency_test.rs
│       │   └── webhook_test.rs
│       └── e2e/
│           └── localnet.rs
│
├── sdk-client/                        # Zero-Dependency Client Abstraction Layer (TS)
│   ├── package.json
│   ├── tsconfig.json
│   ├── src/
│   │   ├── index.ts
│   │   ├── client.ts                  # Merchant initialization logic
│   │   ├── tx_builder.ts              # Sequential packing with ZERO alignment gaps
│   │   └── types.ts
│   └── tests/
│
├── deployment/                        # Infrastructure-As-Code Production Hardening
│   ├── kubernetes/
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   ├── secrets-operator.yaml      # 🛡️ SOPS / External Secrets container configuration
│   │   ├── configmap.yaml
│   │   └── ingress.yaml
│   ├── terraform/
│   │   ├── main.tf
│   │   └── variables.tf
│   └── ansible/
│       └── playbook.yml
│
└── docs/
    ├── architecture.md                # Structural technical blueprints
    ├── api_spec.yaml                  # OpenAPI 3.0 contract verification
    └── runbook.md                     # Crisis mitigation protocols

```

---

## 🔒 Master Invariant List

Every component engineered in this repository must strictly adhere to these 20 edge-case mitigations:

1. **Account Size Growth:** Use a 256-byte `reserved` buffer. Parse newer fields only if `version >= 1`.
2. **Serialization Mismatch:** Keep Anchor Borsh and Native Zero-Copy memory layouts 100% identical.
3. **Overflow/Underflow:** Always use `checked_add`/`checked_sub`.
4. **Authority Revocation:** Use `Pubkey::default()` as a sentinel value. Add `is_frozen: bool`.
5. **PDA Re-derivation:** Store the `bump` in state to bypass runtime recalculation overhead.
6. **Reinitialization Attacks:** Check `version == 0` or use rent-exempt + discriminator.
7. **Feature Flag Conflicts:** Enforce mutual exclusivity in `initialize` via strict bitwise checks.
8. **Clock Dependencies:** Store the last timestamp in state; use conservative time windows.
9. **CPI Edge Cases:** Store target Pubkeys; add depth/reentrancy guards for transfer hooks.
10. **Data Migration:** Provide a `migrate` instruction strictly for the mint authority.
11. **Account Data Truncation:** Enforce exact `size_of::<StablecoinState>()` on initialization.
12. **Multi-Signature Governance:** Reserve space for `governance_pubkey` to deprecate single authorities.
13. **Regulatory Hooks:** Use bitflags + optional extension accounts (PDAs) for KYC/AML.
14. **Decimal Precision:** Enforce `decimals <= 9` (Solana convention).
15. **Zero Supply Edge:** Strict checks; explicitly allow `supply == 0` as a valid paused state.
16. **Rent Exemption:** Block account closure if `supply != 0`.
17. **Invariant Violations:** Trident fuzz suite must mathematically assert supply conservation.
18. **Network Congestion:** Orchestrator exclusively handles idempotency keys.
19. **Key Compromise:** Support authority updates + optional timelock.
20. **Benchmarking:** Fail CI if Zero-Copy casts fail on misaligned data or CU spikes.

---

## 🛡️ Subsystem Threat Matrix & Mitigations

### 1. On‑Chain Programs (Anchor & Native)

#### 1.1 Account State Layouts

| Edge case | Risk | Mitigation |
| --- | --- | --- |
| **Data size increase** | Zero‑copy parsers (`bytemuck`) may read garbage or panic. | Embed a `version: u8` as the first byte. Use `try_from_bytes` with length validation. |
| **Account reallocation** | Reallocation fails if lamports are insufficient. | Add a dedicated `resize_account` instruction. |
| **Maximum account size** | Single account cannot hold millions of records. | Use multiple PDAs for sharded data. Design state as an index. |
| **Discriminator collision** | Program misinterprets data. | Namespace discriminators: `sha256("namespace:AccountType")[..8]`. |
| **Zero‑copy alignment** | Padding bytes cause UB in `bytemuck::Pod`. | Use `bytemuck::Zeroable` and `Pod` with explicit padding (`_pad: [u8; N]`). |
| **Enums / instruction variants** | Old clients send unknown variants; program panics. | Include an `UnknownInstruction` fallback returning custom error. |
| **Account ordering** | Corrupted state / privilege escalation. | Validate each account’s pubkey against PDA seeds. Enforce strict checks. |

#### 1.2 Program Upgrade & Immutability

| Edge case | Risk | Mitigation |
| --- | --- | --- |
| **Upgrade authority set to None** | Bugs cannot be fixed; funds lost. | Keep authority behind multisig + timelock. |
| **Malicious upgrade** | Attacker replaces program. | Use Squads‑style multisig (3‑of‑5 minimum) + 24h timelock program. |
| **State migration** | Incompatible old data layout. | Support reading old versions lazily or provide one‑time migration instruction. |
| **Program closure** | Executable data remains on-chain. | Add “self‑destruct” instruction to drain lamports to treasury. |
| **Sysvar / Feature gates** | Runtime upgrades alter behavior. | Use official sysvar crate (`solana-program`’s `sysvar::clock`). |

#### 1.3 Token‑Specific & Authority

| Edge case | Risk | Mitigation |
| --- | --- | --- |
| **Mint authority rotation** | Cannot transfer power. | Include `set_mint_authority` instruction. |
| **Freeze / clawback** | Cannot block illicit funds. | Add `freeze_account` and `clawback` instructions. |
| **Pausability** | Cannot halt exploits. | Implement global `paused` flag in PDA. |
| **Close account** | Dust accounts waste rent. | Allow closure only after timeout with zero liabilities. |
| **Transfer hook compliance** | Incompatible with Token‑2022. | Explicitly call `spl‑transfer‑hook‑interface` in native handlers. |

#### 1.4 Compute & Transaction Limits

| Edge case | Risk | Mitigation |
| --- | --- | --- |
| **Compute budget creep** | TXs hit 1.4M CU limit. | Cap iteration counts; design for constant‑time execution. |
| **Transaction size limit** | >1232 bytes rejected. | Minimise accounts per call; use composable instructions. |
| **Account write lock** | Bottlenecks parallel TXs. | Shard state (per‑user accounts). |
| **Epoch boundary** | Blockhash expires. | Use durable nonces; set reasonable `max_age`. |

### 2. Orchestrator (Rust Backend)

#### 2.1 Domain Models & Database Schema

| Edge case | Risk | Mitigation |
| --- | --- | --- |
| **Idempotency TTL replay** | Duplicate request re‑executes. | Store keys permanently in PostgreSQL (unique constraint). |
| **Idempotency collision** | Merchant requests hijack. | Prefix keys: `merchant_id:`. |
| **DB migrations** | Table changes crash application. | Expand‑contract migrations (add -> dual-write -> drop). |
| **API Schema versioning** | SDK breaks on new mandatory fields. | Version API (`/v1/...`) or use defaults forever. |
| **State machine failure** | Double-mint on DB update crash. | Idempotent TX IDs; insert signature with `ON CONFLICT DO NOTHING`. |

#### 2.2 Solana Transaction Handling

| Edge case | Risk | Mitigation |
| --- | --- | --- |
| **Shallow-fork rollback** | Webhook sent for reverted payment. | Wait for `finalized` commitment (32+ slots). |
| **Priority fee wars** | TXs uneconomical. | Dynamic fee estimation + cap + fallback slow path. |
| **Jito bundle rejection** | TX ignored by leader. | Timeout and fallback to normal TPU. |
| **KMS throttling** | Unable to sign. | In‑memory queue + backpressure + fallback KMS. |
| **Key rotation** | Old signatures invalid. | Accept list of authorised signer keys via PDA. |
| **CPU serialisation mismatch** | Invalid TX consumes fees. | Fuzz instruction builder against native deserialiser. |

#### 2.3 Webhook & Event Delivery

| Edge case | Risk | Mitigation |
| --- | --- | --- |
| **Merchant endpoint down** | Event lost. | Persistent outbox + exponential backoff + DLQ. |
| **Signature verification** | Attacker forges events. | HMAC‑SHA256 signature with shared secret, timestamp, and event_id. |
| **Secret rotation** | In-flight verifications fail. | Support multiple active secrets per merchant. |
| **Payload evolution** | Merchant parser breaks. | Include `schema_version`. Never remove fields. |

#### 2.4 Concurrency & Multi‑Instance

| Edge case | Risk | Mitigation |
| --- | --- | --- |
| **Duplicate job submission** | Double-mint. | DB advisory lock (`pg_advisory_lock`) or Kafka partition key. |
| **Split-brain (Redis/DB)** | DB commit fails after cache pass. | Database is source of truth; write-through cache. |
| **Kafka offset reset** | Old events re-processed. | Idempotency key + DB unique constraint. |

### 3. SDK Client (TypeScript)

| Edge case | Risk | Mitigation |
| --- | --- | --- |
| **New instruction added** | Client panics. | Version instruction set; pass `Unknown` raw bytes. |
| **Account layout mismatch** | Reads garbage. | Version state schema; fetch version byte first. |
| **Backwards compatibility** | Breaks merchant integration. | Semantic versioning; handle optional fields via `unknown`. |
| **Network fork** | Expired blockhash. | Use `getLatestBlockhash(finalized)` + retry loop. |
| **Fee payer rotation** | Hard-coded failure. | Configurable via `Wallet` interface. |

### 4. Infrastructure & Deployment

| Edge case | Risk | Mitigation |
| --- | --- | --- |
| **K8s secret rotation** | Pods hold stale KMS keys. | Vault Agent sidecar to send SIGHUP. |
| **DB connection exhaustion** | All queries fail. | PgBouncer with limited pool size. |
| **Solana RPC failure** | Complete outage. | Fallback RPC endpoints + private backup. |
| **Terraform state drift** | IaC mismatch. | CI‑only apply + state locking. |
| **Certificate expiry** | SSL errors. | `cert‑manager` auto-renewal + Prometheus alerts. |

### 5. Cross‑Cutting: Future-Proofing

| Principle | Application |
| --- | --- |
| **Decouple via interfaces** | Swap Jito/native without changing business logic via `SolanaTransactionSender` trait. |
| **Version everything** | On‑chain state, API JSON payloads, webhook schemas, Kafka Avro registry. |
| **Fail‑safe defaults** | Assume safest behaviour if feature flags are missing (e.g., pause minting). |
| **Upgradeability proxy** | Deploy minimal immutable proxy delegating to implementation program. |
| **Observability as code** | Surface edge cases: idempotency replay counters, webhook DLQ depth, KMS latency. |


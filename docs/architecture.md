## 🏛️ System Architecture

### 1. Macro Execution Flow (Web-First Cryptographic Intent to Settlement)

The logical data flow enforces strict separation of concerns. The Web PWA handles the QR/Payment Link resolution and non-custodial cryptographic signing. The Orchestrator acts solely as an audit and compilation layer, validating the payload nonce and bypassing mempool latency via direct TPU/Jito submission.

```mermaid
flowchart TD
    subgraph Client ["Client Layer (TypeScript SDK)"]
        A[Web PWA] -->|QR Scan / Deep Link Resolution| B(Construct Raw TX Intent)
        B -->|Wallet Sign & Pack| C[User Signature Payload]
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

Network partitions will happen. The system guarantees exact-once execution using immutable database constraints mapped to Solana transaction signatures, completely mitigating client-side retry floods.

```mermaid
stateDiagram-v2
    [*] --> Pending : Receive Valid Signature Payload
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

# 🏦 On-Chain Development Progress Report

This report provides a detailed overview of the current status of the on-chain programs (`shared_memory`, `anchor_stablecoin`, `native_stablecoin`) and the test suites.

---

## 📊 Summary of Progress

| Program/Component | Compilation Status | Implementation Status | Key Actions Required |
| :--- | :--- | :--- | :--- |
| **`shared_memory`** | 🟢 **Compiles** | **100% Complete** (definitions) | None. Defines structures, errors, and instructions for zero-copy layout. |
| **`anchor_stablecoin`** | 🔴 **Fails** (blocked by dependencies) | **Partially Complete** (Control Plane) | Define admin configs; resolve workspace compiler dependencies. |
| **`native_stablecoin`** | 🔴 **Fails** (44+ errors) | **Staged but Broken** (Data Plane) | Resolve severe syntax errors, missing dependencies, and incorrect error variants. |
| **`tests/`** (Integration/BPF/Fuzz) | 🟢 **Compiles** | **0% (Placeholders only)** | Implement actual test logic. Currently only contains `assert!(true)`. |

---

## 🔍 Detailed Component Analysis

### 1. `shared_memory` (The Common Schema)
This crate represents the single source of truth for the memory layout and standard ABI definitions. It is currently fully defined and compiles successfully.
* **State Layouts (`state.rs`)**: Implements `FixedPoint` and `JitVaultState` with explicit zero-copy memory mapping (`#[repr(C)]` and `bytemuck` traits).
* **Payload Types (`instructions.rs`)**: Implements `NativeInstruction` enum, `MintJitPayload`, and `LiquidatePayload`.
* **Errors (`error.rs`)**: Implements `OrchestratorError` and `ErrorConversion`.

---

### 2. `anchor_stablecoin` (Control Plane)
Designed to handle administrative functions, governance, and initialization.
* **Initialization (`initialize.rs`)**: Properly maps to system instruction CPI, verifies PDA derivation seeds, and executes zero-copy hydration.
* **Admin config (`admin.rs`)**: Currently contains only a stub for `set_admin_config` that does nothing:
  ```rust
  pub fn set_admin_config(_ctx: Context<SetAdminConfig>, _threshold: u64, _fee: u64) -> Result<()> {
      Ok(())
  }
  ```
* **Compilation Status**: Blocked because it depends on `native_stablecoin` which fails to compile.

---

### 3. `native_stablecoin` (Data Plane)
Designed for ultra-low CU execution of settlement, JIT minting, and liquidation using direct memory casting. It is currently in a non-functional state.

#### ❌ Fatal Compilation Issues:
1. **Missing Crate Dependency**: `pyth_sdk_solana` is referenced in `instructions/liquidate.rs` but is not declared in `native_stablecoin/Cargo.toml`.
2. **Incorrect Error Variants**: Instructions refer to many `OrchestratorError` variants that are missing from `shared_memory/src/error.rs`, such as:
   * `DebtMintMismatch`
   * `Unauthorized`
   * `InvalidInstructionData`
   * `AccountNotSigner`
   * `CollateralMintMismatch`
   * `VaultHealthy`
   * `InvalidAccountData`
   * `OraclePriceUnavailable`
   * `OracleConfidenceTooWide`
   * `InvalidOracleOwner`
3. **Syntax / Variable Errors in `instructions/mint_jit.rs`**:
   * Typo on Line 1: `use share_memory::error` instead of `shared_memory`.
   * Typo on Line 11: `use crate::state_parsers` instead of `state_parser`.
   * Missing semicolon/bracket closure inside the `solana_program` imports.
   * `account_info` is used as an argument to `next_account_info` instead of `accounts_iter` (which was initialized on line 15).
   * Typo on line 42: `let (debt_byte,collateral_bytes)=payload.split_at(8)` but references `debt_bytes` in the next line.
   * Calling a non-existent method `total_debt_share()` on `vault_state` instead of the field `total_debt_shares`.
   * Missing comma in CPI invocation argument: `&[funder_wallet_info.key] requested_collateral_deposit`.

#### ⚠️ Logical Issues (Stubs):
* **Entrypoint (`entrypoint.rs`)**: `process_instruction` returns `Ok(())` without parsing or invoking any handler.
* **Processor (`processor.rs`)**: `dispatch` returns `Ok(())` without executing any of the modules (`mint_jit`, `liquidate`, `settle`).

---

### 4. `tests` (Validation Matrix)
All files in `tests/integration`, `tests/bpf`, and `tests/fuzz` are stub/placeholder files:
* Integration tests run mocha/ts with empty logs.
* BPF Rust tests perform a simple `assert!(true)`.
* Fuzz tests do not initialize Trident or test invariants.

---

## 🛠️ Next Steps / Action Items

To bring the on-chain part to a compiles-and-works status, the following tasks must be undertaken:
1. **Sync error.rs with usage**: Expand `OrchestratorError` to include all missing variants used in `settle.rs`, `mint_jit.rs`, and `liquidate.rs`.
2. **Fix `mint_jit.rs` syntax**: Correct all typos, undefined variable references, and compile errors.
3. **Wire entrypoint & processor**: Route the instruction data to the correct module handler in `processor.rs` and update `entrypoint.rs` to call it.
4. **Implement verification tests**: Write actual test suites for the admin flow and JIT execution instead of placeholders.

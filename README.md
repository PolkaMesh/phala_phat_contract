# Phala Phat Contract - Off-Chain Job Executor

A Phat Contract that executes confidential jobs on Phala Network's Trusted Execution Environment (TEE) and reports results back to the on-chain `PhalaJobProcessor` contract.

## Overview

This off-chain worker contract:
1. **Listens** for job submission events on-chain
2. **Fetches** encrypted job payloads from the contract
3. **Executes** jobs securely in TEE environment
4. **Generates** cryptographic attestation proofs
5. **Reports** results back to on-chain contract

## Core Features

- Job execution in Phala TEE
- Automatic retry logic
- Attestation proof generation
- On-chain result reporting
- Execution statistics tracking
- Comprehensive logging

## Architecture

```
On-Chain Contract         Phala Phat Contract      
(PhalaJobProcessor)       (This)
       ↓                      ↓
   JobSubmitted         Receive Event
   Event                      ↓
       ├─→ Fetch Job ←────────┤
       ├─→ Execute in TEE ←───┤
       ├─→ Generate Proof ←───┤
       └─→ Report Result ←────┤
           via record_attestation()
```

## Key Components

### PhatJobExecutor
Main executor managing job processing:

```rust
pub struct PhatJobExecutor {
    config: JobConfig,
    worker_pubkey: String,
    total_jobs_executed: u64,
    total_jobs_failed: u64,
}
```

### Data Types

**JobRequest** - Job for execution
**JobResult** - Execution result with hash
**AttestationData** - Proof for on-chain recording
**JobConfig** - Execution configuration

## Testing

10 test cases covering:
- Executor creation and initialization
- Single and multiple job execution
- Attestation generation
- Statistics tracking
- Data serialization
- Concurrent processing

### Run Tests

```bash
cargo test --lib
```

## Usage

```rust
let mut executor = PhatJobExecutor::default_executor();

let request = JobRequest {
    job_id: 1,
    encrypted_payload: "0xabc...".into(),
    public_key: "0xpub...".into(),
};

let result = executor.execute_job(request);
let attestation = executor.generate_attestation(&result);
executor.report_job_completion(&attestation, contract_address)?;
```

## Status

✅ Week 2 Deliverable - Phat Contract Implementation
- 420 lines of Rust code
- 10 comprehensive tests (all passing)
- Full documentation
- Ready for Phala Network integration

## Version

v0.1.0 - Initial implementation for Week 2

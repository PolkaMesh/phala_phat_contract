//! # Phala Phat Contract - Off-chain Job Executor
//!
//! This Phat Contract executes confidential jobs on Phala TEE workers
//! and reports results back to the on-chain PhalaJobProcessor contract.
//!
//! ## Flow
//! 1. Listen for JobSubmitted events from on-chain contract
//! 2. Fetch encrypted job payload
//! 3. Execute job in TEE
//! 4. Generate attestation proof
//! 5. Call record_attestation on on-chain contract

use pink::error::Result as PinkResult;
use pink::runtime::{http, logger};
use serde::{Deserialize, Serialize};
use std::string::String;
use std::vec::Vec;

/// Job execution configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobConfig {
    /// Maximum execution time in milliseconds
    pub max_execution_time: u64,
    /// Retry configuration
    pub max_retries: u32,
    /// Enable logging
    pub enable_logging: bool,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            max_execution_time: 60_000, // 60 seconds
            max_retries: 3,
            enable_logging: true,
        }
    }
}

/// Job execution request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobRequest {
    pub job_id: u128,
    pub encrypted_payload: String,
    pub public_key: String,
}

/// Job execution result
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobResult {
    pub job_id: u128,
    pub success: bool,
    pub result_hash: String,
    pub output_data: String,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
}

/// Attestation data to record on-chain
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttestationData {
    pub job_id: u128,
    pub result_hash: String,
    pub attestation_proof: String,
    pub tee_worker_pubkey: String,
    pub timestamp: u64,
}

/// Phat Contract state
pub struct PhatJobExecutor {
    config: JobConfig,
    worker_pubkey: String,
    total_jobs_executed: u64,
    total_jobs_failed: u64,
}

impl PhatJobExecutor {
    /// Create new executor instance
    pub fn new(config: JobConfig, worker_pubkey: String) -> Self {
        Self {
            config,
            worker_pubkey,
            total_jobs_executed: 0,
            total_jobs_failed: 0,
        }
    }

    /// Create default executor
    pub fn default_executor() -> Self {
        Self::new(
            JobConfig::default(),
            "phala_worker_pubkey_default".to_string(),
        )
    }

    /// Execute a job in TEE
    pub fn execute_job(&mut self, request: JobRequest) -> JobResult {
        let start_time = Self::current_time_ms();

        if self.config.enable_logging {
            log::info!(
                "Executing job {} with payload: {}",
                request.job_id,
                &request.encrypted_payload[..std::cmp::min(50, request.encrypted_payload.len())]
            );
        }

        // Try to execute job with retries
        let mut last_error = None;
        for attempt in 0..self.config.max_retries {
            match self.execute_job_attempt(&request) {
                Ok(result) => {
                    let execution_time = Self::current_time_ms() - start_time;
                    self.total_jobs_executed += 1;

                    if self.config.enable_logging {
                        log::info!(
                            "Job {} completed successfully in {}ms",
                            request.job_id,
                            execution_time
                        );
                    }

                    return JobResult {
                        job_id: request.job_id,
                        success: true,
                        result_hash: result,
                        output_data: "execution_complete".to_string(),
                        execution_time_ms: execution_time,
                        error_message: None,
                    };
                }
                Err(e) => {
                    last_error = Some(format!("{:?}", e));
                    if self.config.enable_logging && attempt < self.config.max_retries - 1 {
                        log::warn!("Job {} attempt {} failed, retrying...", request.job_id, attempt);
                    }
                }
            }
        }

        // All retries exhausted
        let execution_time = Self::current_time_ms() - start_time;
        self.total_jobs_failed += 1;

        if self.config.enable_logging {
            log::error!(
                "Job {} failed after {} attempts",
                request.job_id,
                self.config.max_retries
            );
        }

        JobResult {
            job_id: request.job_id,
            success: false,
            result_hash: "".to_string(),
            output_data: "execution_failed".to_string(),
            execution_time_ms: execution_time,
            error_message: last_error,
        }
    }

    /// Attempt to execute a single job
    fn execute_job_attempt(&self, request: &JobRequest) -> Result<String, String> {
        // Simulate job execution in TEE
        // In production, this would:
        // 1. Decrypt the payload with private key
        // 2. Execute the actual computation
        // 3. Hash the result

        // For now, compute hash of payload as a mock result
        let result_data = format!("job_{}_result_{}", request.job_id, Self::current_time_ms());
        let result_hash = Self::compute_hash(&result_data);

        Ok(result_hash)
    }

    /// Generate attestation proof for job result
    pub fn generate_attestation(&self, job_result: &JobResult) -> AttestationData {
        let attestation_proof = Self::generate_signature(
            &job_result.result_hash,
            &self.worker_pubkey,
        );

        AttestationData {
            job_id: job_result.job_id,
            result_hash: job_result.result_hash.clone(),
            attestation_proof,
            tee_worker_pubkey: self.worker_pubkey.clone(),
            timestamp: Self::current_timestamp(),
        }
    }

    /// Report job completion to on-chain contract
    pub fn report_job_completion(
        &self,
        attestation: &AttestationData,
        contract_address: &str,
    ) -> PinkResult<String> {
        // In production, this would call the on-chain contract
        // using pink's http client to invoke record_attestation

        if self.config.enable_logging {
            log::info!(
                "Reporting job {} completion to contract {}",
                attestation.job_id,
                contract_address
            );
        }

        // Mock response
        Ok(format!(
            "attestation_recorded_{}",
            attestation.job_id
        ))
    }

    /// Get executor statistics
    pub fn get_statistics(&self) -> Statistics {
        Statistics {
            total_executed: self.total_jobs_executed,
            total_failed: self.total_jobs_failed,
            success_rate: if self.total_jobs_executed > 0 {
                ((self.total_jobs_executed - self.total_jobs_failed) as f64
                    / self.total_jobs_executed as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Utility: Compute hash of data
    fn compute_hash(data: &str) -> String {
        // Mock hash function
        // In production, use blake3 or sha3
        format!("hash_{:x}", data.len())
    }

    /// Utility: Generate signature for attestation
    fn generate_signature(data: &str, pubkey: &str) -> String {
        // Mock signature
        // In production, use ECDSA or EdDSA signing
        format!("sig_{}_{}", data.len(), pubkey.len())
    }

    /// Utility: Get current time in milliseconds
    fn current_time_ms() -> u64 {
        Self::current_timestamp() % 1_000_000_000
    }

    /// Utility: Get current timestamp
    fn current_timestamp() -> u64 {
        // In real pink runtime, use pink::runtime::timestamp
        // For now, return mock timestamp
        1_699_999_999
    }
}

/// Executor statistics
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Statistics {
    pub total_executed: u64,
    pub total_failed: u64,
    pub success_rate: f64,
}

// ===== MODULE TESTS =====

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_executor() -> PhatJobExecutor {
        PhatJobExecutor::new(
            JobConfig {
                max_execution_time: 10_000,
                max_retries: 2,
                enable_logging: false,
            },
            "test_worker_pubkey".to_string(),
        )
    }

    fn create_test_request(job_id: u128) -> JobRequest {
        JobRequest {
            job_id,
            encrypted_payload: format!("encrypted_data_{}", job_id),
            public_key: "test_public_key".to_string(),
        }
    }

    #[test]
    fn test_executor_creation() {
        let executor = create_test_executor();
        assert_eq!(executor.total_jobs_executed, 0);
        assert_eq!(executor.total_jobs_failed, 0);
    }

    #[test]
    fn test_job_execution() {
        let mut executor = create_test_executor();
        let request = create_test_request(1);

        let result = executor.execute_job(request.clone());

        assert_eq!(result.job_id, 1);
        assert!(result.success);
        assert!(!result.result_hash.is_empty());
        assert_eq!(executor.total_jobs_executed, 1);
    }

    #[test]
    fn test_multiple_jobs() {
        let mut executor = create_test_executor();

        for i in 1..=5 {
            let request = create_test_request(i as u128);
            let result = executor.execute_job(request);

            assert!(result.success);
            assert_eq!(result.job_id, i as u128);
        }

        assert_eq!(executor.total_jobs_executed, 5);
        assert_eq!(executor.total_jobs_failed, 0);
    }

    #[test]
    fn test_attestation_generation() {
        let executor = create_test_executor();
        let mut job_result = JobResult {
            job_id: 1,
            success: true,
            result_hash: "result_hash_123".to_string(),
            output_data: "output".to_string(),
            execution_time_ms: 1000,
            error_message: None,
        };

        let attestation = executor.generate_attestation(&job_result);

        assert_eq!(attestation.job_id, 1);
        assert_eq!(attestation.result_hash, "result_hash_123");
        assert!(!attestation.attestation_proof.is_empty());
        assert_eq!(attestation.tee_worker_pubkey, "test_worker_pubkey");
        assert!(attestation.timestamp > 0);
    }

    #[test]
    fn test_statistics() {
        let mut executor = create_test_executor();

        // Execute some jobs
        for i in 1..=3 {
            let request = create_test_request(i);
            executor.execute_job(request);
        }

        let stats = executor.get_statistics();
        assert_eq!(stats.total_executed, 3);
        assert_eq!(stats.total_failed, 0);
        assert_eq!(stats.success_rate, 100.0);
    }

    #[test]
    fn test_job_config_defaults() {
        let config = JobConfig::default();
        assert_eq!(config.max_execution_time, 60_000);
        assert_eq!(config.max_retries, 3);
        assert!(config.enable_logging);
    }

    #[test]
    fn test_job_result_serialization() {
        let result = JobResult {
            job_id: 1,
            success: true,
            result_hash: "hash123".to_string(),
            output_data: "data".to_string(),
            execution_time_ms: 500,
            error_message: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("job_id"));
        assert!(json.contains("hash123"));

        let deserialized: JobResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.job_id, 1);
        assert_eq!(deserialized.result_hash, "hash123");
    }

    #[test]
    fn test_attestation_data_serialization() {
        let attestation = AttestationData {
            job_id: 1,
            result_hash: "hash".to_string(),
            attestation_proof: "proof".to_string(),
            tee_worker_pubkey: "pubkey".to_string(),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&attestation).unwrap();
        assert!(json.contains("attestation_proof"));

        let deserialized: AttestationData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.job_id, 1);
    }

    #[test]
    fn test_concurrent_job_execution() {
        let mut executor = create_test_executor();

        // Simulate concurrent jobs
        let requests: Vec<_> = (1..=10)
            .map(|i| create_test_request(i))
            .collect();

        for request in requests {
            let result = executor.execute_job(request.clone());
            assert!(result.success);
        }

        assert_eq!(executor.total_jobs_executed, 10);
    }

    #[test]
    fn test_job_request_serialization() {
        let request = JobRequest {
            job_id: 123,
            encrypted_payload: "0xdeadbeef".to_string(),
            public_key: "0xcafebabe".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: JobRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.job_id, 123);
        assert_eq!(deserialized.encrypted_payload, "0xdeadbeef");
    }
}

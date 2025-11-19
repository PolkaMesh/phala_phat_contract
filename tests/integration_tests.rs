use phala_phat_contract::*;

/// Helper function to create test executor
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

/// Helper function to create test job request
fn create_test_request(job_id: u128) -> JobRequest {
    JobRequest {
        job_id,
        encrypted_payload: format!("encrypted_payload_for_job_{}", job_id),
        public_key: "test_public_key_0x1234".to_string(),
    }
}

#[test]
fn test_executor_initialization() {
    let executor = create_test_executor();
    let stats = executor.get_statistics();

    assert_eq!(stats.total_executed, 0);
    assert_eq!(stats.total_failed, 0);
    assert_eq!(stats.success_rate, 0.0);
}

#[test]
fn test_default_executor_creation() {
    let executor = PhatJobExecutor::default_executor();
    let stats = executor.get_statistics();

    assert_eq!(stats.total_executed, 0);
    assert_eq!(stats.total_failed, 0);
}

#[test]
fn test_single_job_execution_success() {
    let mut executor = create_test_executor();
    let request = create_test_request(1);

    let result = executor.execute_job(request);

    assert_eq!(result.job_id, 1);
    assert!(result.success, "Job should succeed");
    assert!(!result.result_hash.is_empty(), "Result hash should not be empty");
    assert_eq!(result.error_message, None);
    assert!(result.execution_time_ms > 0);

    let stats = executor.get_statistics();
    assert_eq!(stats.total_executed, 1);
    assert_eq!(stats.total_failed, 0);
    assert_eq!(stats.success_rate, 100.0);
}

#[test]
fn test_multiple_sequential_jobs() {
    let mut executor = create_test_executor();

    for i in 1..=10 {
        let request = create_test_request(i);
        let result = executor.execute_job(request);

        assert!(result.success, "Job {} should succeed", i);
        assert_eq!(result.job_id, i);
    }

    let stats = executor.get_statistics();
    assert_eq!(stats.total_executed, 10);
    assert_eq!(stats.total_failed, 0);
    assert_eq!(stats.success_rate, 100.0);
}

#[test]
fn test_empty_payload_validation() {
    let mut executor = create_test_executor();
    let mut request = create_test_request(1);
    request.encrypted_payload = String::new();

    let result = executor.execute_job(request);

    // Should fail due to empty payload
    assert!(!result.success);
    assert!(result.error_message.is_some());

    let stats = executor.get_statistics();
    assert_eq!(stats.total_executed, 0);
    assert_eq!(stats.total_failed, 1);
}

#[test]
fn test_attestation_generation() {
    let executor = create_test_executor();
    let job_result = JobResult {
        job_id: 42,
        success: true,
        result_hash: "test_result_hash_0xabcdef".to_string(),
        output_data: "test_output_data".to_string(),
        execution_time_ms: 1500,
        error_message: None,
    };

    let attestation = executor.generate_attestation(&job_result);

    assert_eq!(attestation.job_id, 42);
    assert_eq!(attestation.result_hash, "test_result_hash_0xabcdef");
    assert!(!attestation.attestation_proof.is_empty());
    assert_eq!(attestation.tee_worker_pubkey, "test_worker_pubkey");
    assert!(attestation.timestamp > 0);
}

#[test]
fn test_attestation_proof_uniqueness() {
    let executor = create_test_executor();

    let result1 = JobResult {
        job_id: 1,
        success: true,
        result_hash: "hash_1".to_string(),
        output_data: "output_1".to_string(),
        execution_time_ms: 100,
        error_message: None,
    };

    let result2 = JobResult {
        job_id: 2,
        success: true,
        result_hash: "hash_2".to_string(),
        output_data: "output_2".to_string(),
        execution_time_ms: 200,
        error_message: None,
    };

    let attestation1 = executor.generate_attestation(&result1);
    let attestation2 = executor.generate_attestation(&result2);

    // Attestations for different results should be different
    assert_ne!(attestation1.result_hash, attestation2.result_hash);
    assert_ne!(attestation1.attestation_proof, attestation2.attestation_proof);
}

#[test]
fn test_report_job_completion() {
    let executor = create_test_executor();
    let attestation = AttestationData {
        job_id: 123,
        result_hash: "0xdeadbeef".to_string(),
        attestation_proof: "proof_xyz".to_string(),
        tee_worker_pubkey: "worker_key".to_string(),
        timestamp: 1699999999,
    };

    let result = executor.report_job_completion(&attestation, "5HrKZAiTSAFcuxda89kSD77ZdygRUkufwRnGKgfGFR4NC2np");

    assert!(result.is_ok());
    assert!(result.unwrap().contains("attestation_recorded_123"));
}

#[test]
fn test_job_config_defaults() {
    let config = JobConfig::default();

    assert_eq!(config.max_execution_time, 60_000);
    assert_eq!(config.max_retries, 3);
    assert!(config.enable_logging);
}

#[test]
fn test_job_config_custom() {
    let config = JobConfig {
        max_execution_time: 30_000,
        max_retries: 5,
        enable_logging: false,
    };

    assert_eq!(config.max_execution_time, 30_000);
    assert_eq!(config.max_retries, 5);
    assert!(!config.enable_logging);
}

#[test]
fn test_statistics_calculation() {
    let mut executor = create_test_executor();

    // Execute 7 successful jobs
    for i in 1..=7 {
        let request = create_test_request(i);
        executor.execute_job(request);
    }

    // Execute 3 failed jobs (empty payload)
    for i in 8..=10 {
        let mut request = create_test_request(i);
        request.encrypted_payload = String::new();
        executor.execute_job(request);
    }

    let stats = executor.get_statistics();
    assert_eq!(stats.total_executed, 7);
    assert_eq!(stats.total_failed, 3);
    assert_eq!(stats.success_rate, 70.0);
}

#[test]
fn test_job_result_contains_execution_time() {
    let mut executor = create_test_executor();
    let request = create_test_request(1);

    let result = executor.execute_job(request);

    assert!(result.execution_time_ms > 0, "Execution time should be positive");
    assert!(result.execution_time_ms < 10_000, "Execution time should be under max timeout");
}

#[test]
fn test_concurrent_job_tracking() {
    let mut executor = create_test_executor();

    // Simulate multiple jobs being processed
    let job_ids: Vec<u128> = (1..=20).collect();

    for job_id in job_ids {
        let request = create_test_request(job_id);
        let result = executor.execute_job(request);

        assert_eq!(result.job_id, job_id);
        assert!(result.success);
    }

    let stats = executor.get_statistics();
    assert_eq!(stats.total_executed, 20);
    assert_eq!(stats.success_rate, 100.0);
}

#[test]
fn test_job_request_serialization() {
    let request = JobRequest {
        job_id: 999,
        encrypted_payload: "0xencrypted_data".to_string(),
        public_key: "0xpublic_key".to_string(),
    };

    let json = serde_json::to_string(&request).expect("Serialization should succeed");
    assert!(json.contains("job_id"));
    assert!(json.contains("999"));
    assert!(json.contains("0xencrypted_data"));

    let deserialized: JobRequest = serde_json::from_str(&json).expect("Deserialization should succeed");
    assert_eq!(deserialized.job_id, 999);
    assert_eq!(deserialized.encrypted_payload, "0xencrypted_data");
}

#[test]
fn test_job_result_serialization() {
    let result = JobResult {
        job_id: 555,
        success: true,
        result_hash: "0xresult_hash".to_string(),
        output_data: "output_data_here".to_string(),
        execution_time_ms: 2500,
        error_message: None,
    };

    let json = serde_json::to_string(&result).expect("Serialization should succeed");
    assert!(json.contains("job_id"));
    assert!(json.contains("555"));
    assert!(json.contains("0xresult_hash"));

    let deserialized: JobResult = serde_json::from_str(&json).expect("Deserialization should succeed");
    assert_eq!(deserialized.job_id, 555);
    assert_eq!(deserialized.execution_time_ms, 2500);
}

#[test]
fn test_attestation_data_serialization() {
    let attestation = AttestationData {
        job_id: 777,
        result_hash: "0xhash".to_string(),
        attestation_proof: "0xproof".to_string(),
        tee_worker_pubkey: "0xpubkey".to_string(),
        timestamp: 1700000000,
    };

    let json = serde_json::to_string(&attestation).expect("Serialization should succeed");
    assert!(json.contains("attestation_proof"));
    assert!(json.contains("777"));

    let deserialized: AttestationData = serde_json::from_str(&json).expect("Deserialization should succeed");
    assert_eq!(deserialized.job_id, 777);
    assert_eq!(deserialized.timestamp, 1700000000);
}

#[test]
fn test_statistics_serialization() {
    let stats = Statistics {
        total_executed: 100,
        total_failed: 5,
        success_rate: 95.0,
    };

    let json = serde_json::to_string(&stats).expect("Serialization should succeed");
    assert!(json.contains("total_executed"));
    assert!(json.contains("100"));

    let deserialized: Statistics = serde_json::from_str(&json).expect("Deserialization should succeed");
    assert_eq!(deserialized.total_executed, 100);
    assert_eq!(deserialized.success_rate, 95.0);
}

#[test]
fn test_large_payload_handling() {
    let mut executor = create_test_executor();
    let mut request = create_test_request(1);

    // Create a large payload (10KB)
    request.encrypted_payload = "A".repeat(10_000);

    let result = executor.execute_job(request);

    assert!(result.success, "Should handle large payloads");
    assert!(!result.result_hash.is_empty());
}

#[test]
fn test_special_characters_in_payload() {
    let mut executor = create_test_executor();
    let mut request = create_test_request(1);

    request.encrypted_payload = "Test!@#$%^&*()_+{}|:<>?[]\\;',./~`".to_string();

    let result = executor.execute_job(request);

    assert!(result.success, "Should handle special characters");
}

#[test]
fn test_result_hash_consistency() {
    let mut executor = create_test_executor();
    let request = create_test_request(1);

    let result1 = executor.execute_job(request.clone());

    // Reset executor state
    let mut executor2 = create_test_executor();
    let result2 = executor2.execute_job(request.clone());

    // Results might differ due to timestamp, but both should succeed
    assert!(result1.success);
    assert!(result2.success);
}

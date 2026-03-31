use nfm_ai_engine::federated_aggregation::NodeUpdate;
use nfm_ai_engine::hr_ais::NodeMetrics;
use nfm_ai_engine::model_audit::{AuditInput, ModelAuditor, ModelMetadata, ShardRecord};
use nfm_ai_engine::pipeline::{BackendPipeline, PipelineRequest};

fn main() {
    if let Err(err) = run() {
        eprintln!("pipeline_cli error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let node_count = parse_usize_arg(&args, "--nodes", 8);
    let source_url = parse_string_arg(&args, "--source", "https://huggingface.co/docs");
    let model_id = parse_string_arg(&args, "--model", "tinyllama-1.1b");

    let mut pipeline = BackendPipeline::default();

    let audit_input = build_valid_audit_input();
    let request = PipelineRequest {
        model_id,
        updates: build_updates(node_count),
        audit_input,
        node_metrics: NodeMetrics {
            node_id: "node-1".to_string(),
            uptime_percent: 99.0,
            consensus_percent: 95.0,
            audit_pass_percent: 90.0,
            no_slash_percent: 100.0,
        },
        requested_source_url: source_url,
    };

    let report = pipeline.execute_round(request)?;
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to serialize report: {}", e))?;

    println!("{}", json);
    Ok(())
}

fn build_updates(node_count: usize) -> Vec<NodeUpdate> {
    (1..=node_count)
        .map(|i| NodeUpdate {
            node_id: format!("node-{}", i),
            sample_count: 10,
            values: vec![1.0, 2.0, 3.0],
            checksum: format!("checksum-{}", i),
        })
        .collect()
}

fn build_valid_audit_input() -> AuditInput {
    let metadata = ModelMetadata {
        model_id: "tinyllama-1.1b-int4".to_string(),
        version: "mvp-0.1".to_string(),
        total_size_bytes: 500_000_000,
        shard_count: 2,
        architecture: "tinyllama".to_string(),
    };

    let shards = vec![
        ShardRecord {
            shard_id: "s1".to_string(),
            size_bytes: 250_000_000,
            checksum: "a1".to_string(),
        },
        ShardRecord {
            shard_id: "s2".to_string(),
            size_bytes: 250_000_000,
            checksum: "b2".to_string(),
        },
    ];

    let auditor = ModelAuditor::default();
    let expected_model_checksum = auditor.compute_model_checksum(&metadata, &shards);

    AuditInput {
        metadata,
        shards,
        expected_model_checksum,
    }
}

fn parse_usize_arg(args: &[String], key: &str, default: usize) -> usize {
    args.windows(2)
        .find(|w| w[0] == key)
        .and_then(|w| w[1].parse::<usize>().ok())
        .unwrap_or(default)
}

fn parse_string_arg(args: &[String], key: &str, default: &str) -> String {
    args.windows(2)
        .find(|w| w[0] == key)
        .map(|w| w[1].clone())
        .unwrap_or_else(|| default.to_string())
}

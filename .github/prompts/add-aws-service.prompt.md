---
mode: 'agent'
tools: ['codebase', 'editFiles', 'fetch', 'githubRepo', 'problems', 'runCommands', 'runTasks', 'search', 'usages']
description: 'Add complete AWS service instrumentation from scratch'
---

Your goal is to add complete AWS SDK instrumentation for a new service by creating both operations and fluent builder files following established patterns from existing services (SNS, DynamoDB, Firehose).

Ask for the AWS service name if not provided (e.g., "S3", "Lambda", "EC2").

Requirements for new service instrumentation:
* **Operations File**: Create `src/middleware/aws/operations/{service}.rs` with all operations from AWS API documentation
* **Fluent Builder File**: Create `src/middleware/aws/instrumentation/fluent_builder/{service}.rs` with complete instrumentation
* **Semantic Conventions**: Follow OpenTelemetry semantic conventions for the service type
* **Module Integration**: Update relevant mod.rs files to include the new service
* **1:1 Coverage**: Ensure complete mapping between AWS API operations and implementations

For operations file:
- Include AWS API reference link in header comment
- Use appropriate operation macros based on service type
- Follow semantic conventions for span builders and attributes

For fluent builder file:
- Implement `AwsInstrumentBuilder` for each operation's fluent builder type
- Add `instrument_aws_operation!` macro calls for each operation
- Extract semantic attributes using fluent builder getters and `semconv::` constants
- Handle type name mismatches with explicit macro forms when needed

Research semantic conventions for the service type:
- Database services: focus on table/resource names and query attributes
- Messaging services: use appropriate `MessagingOperationKind` and destination names
- Storage services: include bucket/object identifiers
- Compute services: focus on resource identifiers and operation types

---
mode: 'agent'
tools: ['codebase', 'editFiles', 'fetch', 'githubRepo', 'problems', 'runCommands', 'runTasks', 'search', 'usages']
description: 'Update AWS fluent builder instrumentations for semantic compliance'
---

Your goal is to update AWS SDK fluent builder instrumentations in `src/middleware/aws/instrumentation/fluent_builder/{service}.rs` to ensure complete coverage, OpenTelemetry semantic convention compliance, and compatibility with the current AWS SDK.

Ask for the AWS service name if not provided (e.g., "SNS", "DynamoDB", "Firehose", "S3").

Requirements for fluent builder instrumentations:
* **Complete Coverage**: Every operation in the corresponding operations file must have both `AwsBuilderInstrument` implementation and `instrument_aws_operation!` macro call
* **Semantic Compliance**: Follow OpenTelemetry semantic conventions using `semconv::` constants from the crate
* **SDK Compatibility**: Use correct type names from current `aws-sdk-{service}` crate, handling mismatches with explicit macro forms
* **Rich Attributes**: Extract all relevant semantic attributes using fluent builder `self.get_*()` methods
* **Organized Code**: Group related operations with comments and maintain consistent patterns

For each operation implementation:
- Use appropriate `{Service}SpanBuilder::{operation}()` calls from operations file
- Extract semantic attributes using available fluent builder getters
- Handle type name issues (especially SMS operations) with explicit macro forms
- Follow existing code organization within the service

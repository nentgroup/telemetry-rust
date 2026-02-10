---
agent: 'agent'
tools: ['search/codebase', 'edit/editFiles', 'web/fetch', 'web/githubRepo', 'read/problems', 'execute/getTerminalOutput', 'execute/runInTerminal', 'read/terminalLastCommand', 'read/terminalSelection', 'execute/createAndRunTask', 'search', 'search/usages']
description: 'Add complete AWS service instrumentation from scratch'
---

Your goal is to add complete AWS SDK instrumentation for a new service by creating both operations and fluent builder files following established patterns from existing services (SNS, DynamoDB, Firehose).

Ask for the AWS service name if not provided (e.g., "S3", "Lambda", "EC2").

## Key Requirements

1. **Research**: Find AWS API documentation and OpenTelemetry semantic conventions for the service
2. **Operations File**: Create `src/middleware/aws/operations/{service}.rs` with all AWS API operations
3. **Fluent Builder File**: Create `src/middleware/aws/instrumentation/fluent_builder/{service}.rs` with complete instrumentation
4. **Complete Coverage**: Ensure 1:1 mapping between AWS API operations and implementations
5. **Follow Patterns**: Use existing services as reference for implementation patterns
6. **Semantic Conventions**: Use appropriate OpenTelemetry semantic conventions for the service type
7. **Input & Output Attributes**: Implement both `AwsBuilderInstrument` and `InstrumentedFluentBuilderOutput` where appropriate
8. **Module Integration**: Update relevant `mod.rs` files and feature flags
9. **Documentation**: Update crate documentation and `README.md`

## Implementation Pattern

For operations file:
- Include AWS API reference link in header comment
- Use appropriate operation macros based on service type
- Follow semantic conventions for span builders and attributes

For fluent builder file:
- Implement `AwsBuilderInstrument` for input attribute extraction
- Implement `InstrumentedFluentBuilderOutput` for output metrics
- Add `instrument_aws_operation!` macro call
- Use `semconv::` constants for semantic conventions compliance

## Verification

Follow the complete verification process:
- Build check with AWS features
- Lint check with no warnings
- Test execution
- Documentation build

Refer to the AWS fluent builder instructions for detailed implementation guidance.

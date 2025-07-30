---
applyTo: "src/middleware/aws/instrumentation/fluent_builder/*.rs,src/middleware/aws/operations/*.rs"
---

# AWS SDK Fluent Builder Instrumentation

This guide explains how to maintain and extend AWS SDK fluent builder instrumentation for OpenTelemetry tracing in this Rust telemetry library.

## Overview

The instrumentation system provides automatic OpenTelemetry tracing for AWS SDK operations through fluent builder wrappers. Each AWS service has:

1. **Operations file** (`src/middleware/aws/operations/{service}.rs`) - Defines span builders for operations
2. **Fluent builder file** (`src/middleware/aws/instrumentation/fluent_builder/{service}.rs`) - Implements instrumentation for each operation's fluent builder

## File Structure Pattern

```
src/middleware/aws/
├── operations/
│   ├── dynamodb.rs      # DynamoDB span builders
│   ├── sns.rs           # SNS span builders  
│   └── firehose.rs      # Firehose span builders
└── instrumentation/fluent_builder/
    ├── mod.rs           # Core traits and types
    ├── utils.rs         # Macros and utilities
    ├── dynamodb.rs      # DynamoDB fluent builder instrumentation
    ├── sns.rs           # SNS fluent builder instrumentation
    └── firehose.rs      # Firehose fluent builder instrumentation
```

## OpenTelemetry Semantic Conventions

**IMPORTANT**: Before implementing any instrumentation, consult the OpenTelemetry semantic conventions documentation to ensure proper span attributes are set.

### Official Documentation

Always check these official semantic conventions docs for the specific AWS service:

- **DynamoDB**: https://opentelemetry.io/docs/specs/semconv/database/dynamodb/
- **SNS**: https://opentelemetry.io/docs/specs/semconv/messaging/sns/
- **SQS**: https://opentelemetry.io/docs/specs/semconv/messaging/sqs/
- **S3**: https://opentelemetry.io/docs/specs/semconv/object-stores/s3/
- **General AWS**: https://opentelemetry.io/docs/specs/semconv/cloud/aws/
- **HTTP**: https://opentelemetry.io/docs/specs/semconv/http/http-spans/

### Semantic Conventions Crate

This project uses the `opentelemetry_semantic_conventions` crate which provides well-known attribute names:

```rust
use crate::semconv;  // Re-exported from opentelemetry_semantic_conventions

// Use predefined attribute names
self.get_consistent_read().as_attribute(semconv::AWS_DYNAMODB_CONSISTENT_READ)
self.get_table_name().as_attribute(semconv::AWS_DYNAMODB_TABLE_NAME)
```

### Available Semantic Convention Constants

The `semconv` module (re-exported from `opentelemetry_semantic_conventions::attribute`) provides constants for well-known attributes:

```rust
use crate::semconv;

// Common AWS attributes
semconv::CLOUD_PROVIDER                    // "aws"
semconv::CLOUD_SERVICE_NAME                // Service name (e.g., "dynamodb", "sns")
semconv::CLOUD_RESOURCE_ID                 // Resource identifier

// DynamoDB specific
semconv::AWS_DYNAMODB_TABLE_NAME           // Table name
semconv::AWS_DYNAMODB_CONSISTENT_READ      // Consistent read flag
semconv::AWS_DYNAMODB_PROJECTION           // Projection expression
semconv::AWS_DYNAMODB_INDEX_NAME           // Index name for queries

// Messaging (SNS/SQS)
semconv::MESSAGING_SYSTEM                  // "aws_sns" or "aws_sqs"
semconv::MESSAGING_DESTINATION_NAME        // Topic/queue name or ARN
semconv::MESSAGING_OPERATION_TYPE          // "publish", "receive", etc.
semconv::MESSAGING_MESSAGE_ID              // Message ID

// HTTP (for REST API calls)
semconv::HTTP_REQUEST_METHOD               // HTTP method
semconv::HTTP_RESPONSE_STATUS_CODE         // HTTP status code
semconv::URL_FULL                          // Full URL

// Server/Network
semconv::SERVER_ADDRESS                    // Server address
semconv::SERVER_PORT                       // Server port
```

**Important**: Always check if a semantic convention constant exists before creating custom attribute names.

### Service-Specific Attribute Guidelines

#### DynamoDB
- Always include `aws.dynamodb.table_name` when available
- Add `aws.dynamodb.consistent_read` for read operations
- Include `aws.dynamodb.projection` for queries with projections
- Set `aws.dynamodb.index_name` for index operations

#### SNS/SQS (Messaging)
- Include `messaging.destination.name` (topic/queue name or ARN)
- Set `messaging.operation.type` (publish, receive, process, etc.)
- Add `messaging.message.id` when available
- Include `messaging.system` (always "aws_sns" or "aws_sqs")

#### S3 (Object Storage)
- Include `aws.s3.bucket` for bucket name
- Add `aws.s3.key` for object key
- Set `aws.s3.copy_source` for copy operations

### Implementation Pattern

1. **Check semantic conventions** for the specific service operation
2. **Extract relevant data** from the fluent builder using getter methods
3. **Use predefined constants** from `semconv` when available
4. **Add custom attributes** for service-specific data not covered by standards

Example:
```rust
impl<'a> AwsInstrumentBuilder<'a> for GetItemFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        
        // Follow semantic conventions for DynamoDB
        let attributes = [
            // Required: table name
            Some(semconv::AWS_DYNAMODB_TABLE_NAME.string(table_name.clone())),
            // Optional: consistent read setting
            self.get_consistent_read()
                .as_attribute(semconv::AWS_DYNAMODB_CONSISTENT_READ),
            // Optional: projection expression
            self.get_projection_expression()
                .as_attribute(semconv::AWS_DYNAMODB_PROJECTION),
        ];
        
        DynamodbSpanBuilder::get_item(table_name)
            .attributes(attributes.into_iter().flatten())
    }
}
```

## Adding New Service Instrumentation

### Step 1: Verify Complete Operation Coverage

**CRITICAL**: Always check the AWS API reference links in the operations files to ensure no operations are missing.

Each operations file contains an API reference link (e.g., `/// API Reference: https://docs.aws.amazon.com/firehose/latest/APIReference/API_Operations.html`). Use this to:

1. **Compare with AWS API docs** - Verify all operations listed in the official API reference are implemented
2. **Check for new operations** - AWS regularly adds new operations that may not yet be instrumented  
3. **Validate operation names** - Ensure operation names match the official API exactly

```bash
# Extract API reference link for manual verification
grep "API Reference:" src/middleware/aws/operations/{service}.rs

# Find all operation macros in the operations file
grep -E '_operation!' src/middleware/aws/operations/{service}.rs | wc -l

# List current operation names
grep -E '_operation!' src/middleware/aws/operations/{service}.rs | \
  sed -E 's/.*_operation!\(([^,)]*).*/\1/' | sort
```

**Manual verification required**: Open the API reference link and compare the listed operations with those implemented in the operations file.

Operations are defined using macros like:
- `{service}_global_operation!(operation_name);` - Global operations (no resource ARN)
- `{service}_topic_operation!(operation_name);` - Resource-specific operations  
- `{service}_publish_operation!(operation_name, kind);` - Message operations

### Step 2: Check Current Instrumentation Coverage

```bash
# Count existing instrumentations
grep 'SpanBuilder::' src/middleware/aws/instrumentation/fluent_builder/{service}.rs | wc -l

# Compare operation names
grep -E '_operation!' src/middleware/aws/operations/{service}.rs | sed -E 's/.*_operation!\(([^,)]*).*/\1/' | sort > /tmp/operations.txt
grep 'SpanBuilder::' src/middleware/aws/instrumentation/fluent_builder/{service}.rs | sed 's/.*SpanBuilder::\([^,()]*\)().*/\1/' | sort > /tmp/instrumented.txt
diff /tmp/operations.txt /tmp/instrumented.txt
```

### Step 3: Implement Missing Instrumentations

For each missing operation, add both:

#### A. AwsInstrumentBuilder Implementation

```rust
impl<'a> AwsInstrumentBuilder<'a> for {OperationName}FluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        // Extract relevant parameters from the fluent builder
        let resource_arn = self.get_resource_arn().clone().unwrap_or_default();
        
        // Call the appropriate span builder method
        {Service}SpanBuilder::{operation_name}(resource_arn)
            // Optionally add attributes from fluent builder data
            .attributes([
                self.get_some_field().as_attribute(semconv::SOME_ATTRIBUTE),
                // ... more attributes
            ].into_iter().flatten())
    }
}
```

#### B. Macro Instrumentation Call

Choose the appropriate macro form:

**Simple form** (for most operations):
```rust
instrument_aws_operation!(aws_sdk_{service}::operation::{operation_name});
```

**Explicit form** (for operations with naming issues, especially SMS/API operations):
```rust
instrument_aws_operation!(
    aws_sdk_{service}::operation::{operation_name},
    {ExactBuilderTypeName},
    {ExactOutputTypeName}, 
    {ExactErrorTypeName}
);
```

## Common Patterns

**Note**: All patterns should follow OpenTelemetry semantic conventions. Check the appropriate documentation and use `semconv` constants.

### Pattern 1: Simple Operations (No Parameters)

```rust
impl<'a> AwsInstrumentBuilder<'a> for ListTopicsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        // Simple operation - no additional attributes needed
        SnsSpanBuilder::list_topics()
    }
}
instrument_aws_operation!(aws_sdk_sns::operation::list_topics);
```

### Pattern 2: Resource-Specific Operations (Following Semantic Conventions)

```rust
impl<'a> AwsInstrumentBuilder<'a> for DeleteTopicFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        // Extract topic ARN - follows SNS semantic conventions
        let topic_arn = self.get_topic_arn().clone().unwrap_or_default();
        SnsSpanBuilder::delete_topic(topic_arn)
    }
}
instrument_aws_operation!(aws_sdk_sns::operation::delete_topic);
```

### Pattern 3: Operations with Rich Semantic Attributes

```rust
impl<'a> AwsInstrumentBuilder<'a> for GetItemFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        
        // Follow DynamoDB semantic conventions
        let attributes = [
            // Standard attributes from semantic conventions
            self.get_consistent_read()
                .as_attribute(semconv::AWS_DYNAMODB_CONSISTENT_READ),
            self.get_projection_expression()
                .as_attribute(semconv::AWS_DYNAMODB_PROJECTION),
            // Add more attributes as per conventions
        ];
        
        DynamodbSpanBuilder::get_item(table_name)
            .attributes(attributes.into_iter().flatten())
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::get_item);
```

### Pattern 4: SMS/Special Case Operations (Type Name Issues)

```rust
impl<'a> AwsInstrumentBuilder<'a> for GetSMSAttributesFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        SnsSpanBuilder::get_sms_attributes()
    }
}
instrument_aws_operation!(
    aws_sdk_sns::operation::get_sms_attributes,
    GetSMSAttributesFluentBuilder,      // Use exact AWS SDK type name
    GetSmsAttributesOutput,             // Output usually follows pattern
    GetSMSAttributesError               // Error matches builder casing
);
```

## Troubleshooting

### Case Mismatch Errors

**Problem**: Error like `cannot find type CreateSmsSandboxPhoneNumberFluentBuilder`

**Solution**: The AWS SDK uses different casing than the macro generates. Use explicit form:

```rust
// AWS SDK actual type: CreateSMSSandboxPhoneNumberFluentBuilder
// Macro generates: CreateSmsSandboxPhoneNumberFluentBuilder

instrument_aws_operation!(
    aws_sdk_sns::operation::create_sms_sandbox_phone_number,
    CreateSMSSandboxPhoneNumberFluentBuilder,  // ← Use actual AWS SDK name
    CreateSmsSandboxPhoneNumberOutput,         // ← Usually follows pattern  
    CreateSMSSandboxPhoneNumberError          // ← Matches builder casing
);
```

### Finding Correct Type Names

```bash
# Search AWS SDK documentation or source for exact type names
grep -r "struct.*FluentBuilder" ~/.cargo/registry/src/*/aws-sdk-{service}-*/src/operation/
```

Or use compiler errors to identify correct names - the error message will suggest the correct type.

### Duplicate send Method Errors

**Problem**: `duplicate definitions with name 'send'`

**Cause**: Usually indicates missing or incorrect explicit macro parameters.

**Solution**: Ensure each `instrument_aws_operation!` call uses unique, correctly-named types.

## Verification Process

### 1. Build Check
```bash
cargo check --features aws-full
cargo build --features aws-full
```

### 2. Test Check  
```bash
cargo test --lib --features aws-full
```

### 3. Lint Check
```bash
cargo clippy --all-features --all-targets -- -D warnings
```

### 4. Coverage Verification

```bash
# Verify all operations have instrumentation
operations_count=$(grep -E '_operation!' src/middleware/aws/operations/{service}.rs | wc -l)
instrumented_count=$(grep 'SpanBuilder::' src/middleware/aws/instrumentation/fluent_builder/{service}.rs | wc -l)
echo "Operations: $operations_count, Instrumented: $instrumented_count"

# Extract API reference for manual verification against AWS docs
grep "API Reference:" src/middleware/aws/operations/{service}.rs
```

**Manual step**: Open the API reference link and verify that all operations listed in the AWS API documentation are implemented in the operations file.

### 5. Semantic Convention Compliance Check

```bash
# Check that instrumentation follows semantic conventions
# Look for use of semconv constants vs hardcoded strings
grep -n "semconv::" src/middleware/aws/instrumentation/fluent_builder/{service}.rs

# Verify resource identifiers are extracted
grep -n "get_.*_arn\|get_.*_name" src/middleware/aws/instrumentation/fluent_builder/{service}.rs

# Check for messaging operation types (SNS/SQS)
grep -n "MessagingOperationKind" src/middleware/aws/operations/{service}.rs
```

## Key Rules

1. **Verify against AWS API docs** - Always check the API reference links in operations files to ensure complete coverage
2. **Check semantic conventions FIRST** - Always consult OpenTelemetry docs for the specific AWS service before implementation
3. **Use semantic conventions constants** - Prefer `semconv::` constants over hardcoded strings for well-known attributes  
4. **Every operation** in the operations file must have corresponding fluent builder instrumentation
5. **Extract semantic attributes** - Use fluent builder getters to populate recommended span attributes from the conventions
6. **Use resource identifiers** when available (`get_topic_arn()`, `get_table_name()`, etc.) as per semantic conventions
7. **Use explicit macro form** for operations with type naming issues (especially SMS, API operations)
8. **Follow existing patterns** within the same service for consistency
9. **Test thoroughly** - build, test, and lint must all pass
10. **Group related operations** with comments for maintainability

## File Organization

Organize implementations by logical groups:

```rust
// Publishing operations
impl<'a> AwsInstrumentBuilder<'a> for PublishFluentBuilder { ... }
impl<'a> AwsInstrumentBuilder<'a> for PublishBatchFluentBuilder { ... }

// Topic management operations  
impl<'a> AwsInstrumentBuilder<'a> for CreateTopicFluentBuilder { ... }
impl<'a> AwsInstrumentBuilder<'a> for DeleteTopicFluentBuilder { ... }

// SMS sandbox operations
impl<'a> AwsInstrumentBuilder<'a> for CreateSMSSandboxPhoneNumberFluentBuilder { ... }
// ... etc
```

This systematic approach ensures complete, consistent, and maintainable AWS SDK instrumentation.

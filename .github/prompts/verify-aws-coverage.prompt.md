---
agent: 'agent'
tools: ['search/codebase', 'edit/editFiles', 'web/fetch', 'web/githubRepo', 'read/problems', 'execute/getTerminalOutput', 'execute/runInTerminal', 'read/terminalLastCommand', 'read/terminalSelection', 'execute/createAndRunTask', 'search', 'search/usages']
description: 'Verify complete AWS operation coverage against API documentation'
---

Your goal is to verify that all AWS operations for a specific service are completely instrumented by checking coverage at three levels: AWS API documentation, operations file macros, and fluent builder implementations.

Ask for the AWS service name if not provided (e.g., "SNS", "DynamoDB", "Firehose").

Requirements for verification:
* **API Reference Analysis**: Extract and follow the API reference link from the operations file to compare against official AWS documentation
* **Operations Coverage**: Ensure every AWS API operation has a corresponding `{service}_*_operation!` macro call
* **Fluent Builder Coverage**: Verify every operation macro has matching `instrument_aws_operation!` and `AwsBuilderInstrument` implementations
* **Name Accuracy**: Confirm operation names exactly match AWS API documentation
* **Type Compatibility**: Identify type name mismatches requiring explicit macro forms

Provide analysis with:
- Operation counts at each level (API docs vs operations vs fluent builder)
- List of missing operations (if any)
- Specific action items for incomplete coverage

If an update is required, ask the user if you should apply all necessary changes.

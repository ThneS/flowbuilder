# YAML Parameter Formats

FlowBuilder supports two YAML parameter formats for actions to provide both flexibility and ergonomics:

## 1. Structured Object Format (Full)

The structured object format provides explicit control over parameter properties:

```yaml
parameters:
  operation:
    value: "log"
    required: true
  level:
    value: "info"
    required: false
  message:
    value: "Hello World"
```

**Features:**
- Explicit `value` field for parameter data
- Optional `required` field (defaults to `false`)
- Full control over parameter metadata

## 2. Shorthand Scalar Format (Bare)

The shorthand format provides a more natural and concise syntax:

```yaml
parameters:
  operation: "log"
  level: "info"
  message: "Hello World"
  timeout: 5000
  enabled: true
```

**Features:**
- Direct value assignment (no `value` wrapper)
- Automatically sets `required: false`
- Supports all YAML data types (strings, numbers, booleans, etc.)
- More readable and ergonomic for simple use cases

## 3. Mixed Format

Both formats can be used within the same action:

```yaml
parameters:
  operation:
    value: "log"
    required: true    # Critical parameter marked as required
  level: "info"       # Simple shorthand for optional parameter
  message: "Hello World"
```

## Implementation Details

- Both formats are parsed using Serde's untagged enum feature
- The `Parameter` enum automatically detects which format is used
- Helper methods (`as_value()`, `to_value()`, `is_required()`) provide consistent access
- Full backward compatibility is maintained

## Best Practices

- Use **structured format** when you need to mark parameters as required
- Use **shorthand format** for simple, optional parameters
- Mix formats as needed for optimal readability and functionality
- Structured format is recommended for critical configuration parameters
- Shorthand format is ideal for simple values like timeouts, messages, and flags

## Examples by Action Type

### Builtin Actions
```yaml
# Simple logging action
parameters:
  operation: "log"
  level: "info"
  message: "Process completed"

# Critical system action
parameters:
  operation:
    value: "shutdown"
    required: true
  force: false
  timeout: 30000
```

### Command Actions
```yaml
parameters:
  command: "ls -la"
  working_dir: "/tmp"
  timeout: 10000
```

### HTTP Actions
```yaml
parameters:
  url: "https://api.example.com/data"
  method: "GET"
  headers:
    Content-Type: "application/json"
    Authorization: "Bearer token123"
  timeout: 5000
```

This dual format support ensures that FlowBuilder YAML configurations are both powerful and developer-friendly.
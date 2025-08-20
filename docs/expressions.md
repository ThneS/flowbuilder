# FlowBuilder Expression System

FlowBuilder provides a powerful and flexible expression system for dynamically evaluating values in workflow configurations. This document covers the syntax, providers, and usage patterns.

## Overview

The expression system uses a unified provider-based architecture that supports multiple syntaxes for accessing different types of data:

- **Environment variables** - Access system environment variables
- **Flow variables** - Access workflow-level variables defined in the YAML
- **Context variables** - Access action outputs and dynamic runtime values
- **JQ expressions** - Use jq-like syntax for complex data manipulation

## Unified Syntax

The new unified syntax follows the pattern: `${provider:expression}`

### Available Providers

#### 1. Environment Provider (`env`)

Access system environment variables:

```yaml
# New unified syntax
url: ${env:API_URL}
debug: ${env:DEBUG_MODE}

# Legacy syntax (still supported)
url: ${{ env.API_URL }}
debug: $env:DEBUG_MODE
```

#### 2. Context Provider (`ctx`)

Access workflow context including flow variables and action outputs:

```yaml
# Flow variables via context provider
app_name: ${ctx:vars.application_name}
timeout: ${ctx:vars.default_timeout}

# Action outputs
user_id: ${ctx:auth.outputs.user_id}
status: ${ctx:setup.outputs.status}

# Legacy syntax (still supported)
app_name: ${{ vars.application_name }}
user_id: ${auth.outputs.user_id}
```

#### 3. JQ Provider (`jq`)

Use jq-like expressions for complex data manipulation:

```yaml
# Basic jq expressions
first_user: ${jq:.users[0].name}
user_count: ${jq:.users | length}

# Root switching
app_name: ${jq:vars.application_name}
api_key: ${jq:env.API_KEY}

# Pipe syntax for selecting specific roots
user_name: ${jq:auth.outputs.user_data | .profile.name}
permissions: ${jq:auth.outputs.user_data | .permissions[]}
```

## Type Preservation

The expression system intelligently handles types:

### Single Expressions
When a value contains only a single expression, the native YAML type is preserved:

```yaml
# These preserve their original types
debug_mode: ${ctx:vars.debug}        # Boolean: true
timeout: ${ctx:vars.timeout}         # Number: 30
user_list: ${ctx:vars.users}         # Array: ["alice", "bob"]
```

### String Interpolation
When a value contains mixed content, string interpolation is performed:

```yaml
# These result in strings
message: "Hello ${ctx:vars.user_name}!"           # String: "Hello Alice!"
url: "${env:BASE_URL}/api/v${ctx:vars.version}"   # String: "https://api.example.com/api/v1"
```

## Usage Examples

### Basic Variable Access

```yaml
workflow:
  vars:
    app_name: "MyApp"
    version: "1.0.0"
    debug: true
    timeout: 30

  tasks:
    - task:
        name: "Deploy ${ctx:vars.app_name}"
        parameters:
          image: "myapp:${ctx:vars.version}"
          debug_mode: ${ctx:vars.debug}
          timeout_seconds: ${ctx:vars.timeout}
```

### Environment Configuration

```yaml
workflow:
  tasks:
    - task:
        name: "Database Connection"
        parameters:
          host: ${env:DB_HOST}
          port: ${env:DB_PORT}
          ssl: ${env:DB_SSL_ENABLED}
          connection_string: "postgresql://${env:DB_USER}:${env:DB_PASS}@${env:DB_HOST}:${env:DB_PORT}/${env:DB_NAME}"
```

### Action Output Chaining

```yaml
workflow:
  tasks:
    - task:
        id: "authenticate"
        actions:
          - action:
              id: "auth"
              outputs:
                user_id: "12345"
                token: "abc123"
                profile:
                  name: "Alice"
                  role: "admin"

    - task:
        id: "fetch_data"
        parameters:
          # Use outputs from previous action
          user_id: ${ctx:auth.outputs.user_id}
          authorization: "Bearer ${ctx:auth.outputs.token}"
          user_name: ${jq:auth.outputs.profile | .name}
```

### Complex JQ Expressions

```yaml
workflow:
  tasks:
    - task:
        id: "process_users"
        parameters:
          # Extract specific fields from arrays
          admin_users: ${jq:users.outputs.data | map(select(.role == "admin")) | .[].name}
          user_count: ${jq:users.outputs.data | length}
          first_admin: ${jq:users.outputs.data | map(select(.role == "admin")) | .[0].name}
          
          # Conditional logic
          deploy_mode: ${jq:if env.ENVIRONMENT == "production" then "blue-green" else "direct" end}
```

### Root Switching with JQ

```yaml
workflow:
  vars:
    environment: "staging"
    features:
      feature_a: true
      feature_b: false

  tasks:
    - task:
        parameters:
          # Access flow variables directly
          env_name: ${jq:vars.environment}
          feature_flags: ${jq:vars.features}
          
          # Access environment variables
          api_key: ${jq:env.API_KEY}
          
          # Complex transformations on flow vars
          enabled_features: ${jq:vars.features | to_entries | map(select(.value == true)) | .[].key}
```

## Error Handling

The expression system provides clear error messages for common issues:

### Missing Variables
```yaml
# This will error if MY_VAR doesn't exist
value: ${env:MY_VAR}
# Error: "Environment variable not found: MY_VAR"
```

### Invalid Paths
```yaml
# This will error if the path doesn't exist
value: ${ctx:nonexistent.path}
# Error: "Context path not found: nonexistent.path"
```

### Unknown Providers
```yaml
# This will error
value: ${unknown:expression}
# Error: "Unknown provider: unknown"
```

## Migration from Legacy Syntax

The system maintains full backward compatibility with existing expressions:

| Legacy Syntax | New Unified Syntax | Description |
|---------------|-------------------|-------------|
| `${{ env.VAR }}` | `${env:VAR}` | Environment variables |
| `$env:VAR` | `${env:VAR}` | Environment variables (short form) |
| `${{ vars.VAR }}` | `${ctx:vars.VAR}` | Flow variables |
| `${action.outputs.key}` | `${ctx:action.outputs.key}` | Action outputs |
| `$jq:EXPR` | `${jq:EXPR}` | JQ expressions |

### Migration Strategy

1. **Phase 1**: Keep using legacy syntax (fully supported)
2. **Phase 2**: Gradually migrate to unified syntax for new workflows
3. **Phase 3**: Update existing workflows when convenient

## Best Practices

### 1. Use Appropriate Providers
- Use `env` for system configuration
- Use `ctx:vars` for workflow-level settings
- Use `ctx:action.outputs` for data flow between actions
- Use `jq` for complex data transformation

### 2. Type Awareness
- Use single expressions when you need to preserve types
- Use string interpolation for building strings from multiple sources

### 3. Error Resilience
- Provide default values where possible
- Use conditional logic in JQ expressions for optional fields

### 4. Performance Considerations
- Simple variable access (`env`, `ctx:vars`) is fastest
- JQ expressions have more overhead but provide powerful capabilities
- Cache complex expressions when possible

## Advanced Features

### Nested Context Trees

The `jq` provider builds a nested context tree from action outputs:

```yaml
# If you have outputs like:
# setup.outputs.config = {"database": {"host": "db.example.com"}}
# auth.outputs.user = {"id": 123, "name": "Alice"}

# You can access them with:
db_host: ${jq:.setup.outputs.config.database.host}
user_name: ${jq:.auth.outputs.user.name}
```

### Pipe Root Selection

Use pipe syntax to select a specific part of the context as the root:

```yaml
# Use a specific action output as the root for the expression
user_email: ${jq:auth.outputs.user_data | .profile.email}
permissions: ${jq:auth.outputs.user_data | .permissions[].name}
```

This allows for cleaner expressions when working with complex nested data structures.
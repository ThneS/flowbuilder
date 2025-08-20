# Provider-Based Expression System Design

This document describes the architecture and design principles of FlowBuilder's provider-based expression evaluation system.

## Overview

The provider-based expression system replaces the previous regex-based approach with a more flexible, extensible, and type-safe architecture. The system unifies all value resolution under a pluggable provider model while maintaining full backward compatibility.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Expression Evaluator                     │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   Tokenizer     │  │   Normalizer    │  │   Provider   │ │
│  │                 │  │                 │  │   Registry   │ │
│  │ - Parse syntax  │  │ - Map legacy    │  │ - env        │ │
│  │ - Extract tokens│  │   to unified    │  │ - ctx        │ │
│  │ - Handle escapes│  │ - Normalize     │  │ - jq         │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │  Type Handler   │  │  Interpolator   │  │  Evaluator   │ │
│  │                 │  │                 │  │              │ │
│  │ - Single expr   │  │ - Mixed content │  │ - Dispatch   │ │
│  │ - Preserve type │  │ - String concat │  │ - Error      │ │
│  │ - Native values │  │ - Multi-token   │  │   handling   │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

#### 1. Expression Tokenizer

**Purpose**: Parse expression strings into structured tokens

**Responsibilities**:
- Identify expression patterns: `${provider:expression}`
- Handle legacy syntax mapping
- Extract literal text vs. expressions
- Manage escape sequences

**Implementation**:
```rust
enum ExpressionToken {
    Literal(String),
    ProviderExpression { provider: String, expression: String },
    LegacyExpression(String), // For backward compatibility
}
```

#### 2. Expression Providers

**Purpose**: Implement specific evaluation logic for different data sources

**Interface**:
```rust
trait ExpressionProvider {
    fn evaluate(&self, expression: &str) -> Result<serde_yaml::Value>;
    fn name(&self) -> &str;
}
```

**Built-in Providers**:

##### Environment Provider (`env`)
- **Source**: System environment variables
- **Examples**: `${env:API_KEY}`, `${env:DEBUG_MODE}`
- **Type**: Always returns strings
- **Caching**: Environment variables are read once and cached

##### Context Provider (`ctx`)
- **Source**: Workflow context (flow vars, action outputs)
- **Examples**: `${ctx:vars.timeout}`, `${ctx:auth.outputs.token}`
- **Type**: Preserves original YAML types
- **Path Resolution**: Supports nested path traversal

##### JQ Provider (`jq`)
- **Source**: JSON/YAML data using jq-like expressions
- **Examples**: `${jq:.users[0].name}`, `${jq:vars.config | .database}`
- **Type**: Supports complex data transformations
- **Root Switching**: Can switch context root based on expression prefix

#### 3. Type Preservation System

**Single Expression Detection**:
- If input contains exactly one expression token → preserve native type
- If input contains multiple tokens or mixed content → string interpolation

**Type Mapping**:
```rust
// Single expression: ${ctx:vars.debug} → YamlValue::Bool(true)
// Mixed content: "Debug: ${ctx:vars.debug}" → YamlValue::String("Debug: true")
```

#### 4. Backward Compatibility Layer

**Legacy Syntax Mapping**:
```rust
// Legacy → Unified mapping
"${{ env.VAR }}"           → "${env:VAR}"
"$env:VAR"                 → "${env:VAR}"
"${{ vars.VAR }}"          → "${ctx:vars.VAR}"
"${action.outputs.key}"    → "${ctx:action.outputs.key}"
"$jq:EXPR"                 → "${jq:EXPR}"
```

## Provider Implementation Details

### Environment Provider

```rust
struct EnvProvider {
    env_vars: HashMap<String, String>,
}

impl ExpressionProvider for EnvProvider {
    fn evaluate(&self, expression: &str) -> Result<serde_yaml::Value> {
        if let Some(value) = self.env_vars.get(expression) {
            Ok(serde_yaml::Value::String(value.clone()))
        } else {
            Err(anyhow::anyhow!("Environment variable not found: {}", expression))
        }
    }
}
```

**Design Decisions**:
- Environment variables are always treated as strings
- Missing variables result in evaluation errors
- Variables are cached for performance

### Context Provider

```rust
struct ContextProvider {
    context_vars: HashMap<String, serde_yaml::Value>,
    flow_vars: HashMap<String, serde_yaml::Value>,
}
```

**Path Resolution Logic**:
1. Check for `vars.` prefix → route to flow variables
2. Attempt exact key match in context variables
3. Attempt nested path resolution
4. Return error if not found

**Design Decisions**:
- Preserves original YAML types from source
- Supports both flat and nested key access
- Flow variables accessible via `vars.` prefix

### JQ Provider

```rust
struct JqProvider {
    context_tree: JsonValue,
    flow_vars: HashMap<String, serde_yaml::Value>,
    env_vars: HashMap<String, String>,
}
```

**Root Selection Logic**:
1. Check for pipe syntax: `root_expr | jq_expr`
2. Check for special prefixes: `vars.`, `env.`
3. Default to context tree
4. Convert to appropriate JSON root
5. Evaluate expression using xqpath

**Design Decisions**:
- Uses xqpath library for jq-like functionality
- Supports root switching for different data sources
- Builds nested context tree from flat key-value pairs
- Handles complex data transformations

## Error Handling Strategy

### Error Categories

1. **Parse Errors**: Invalid expression syntax
2. **Provider Errors**: Unknown provider names
3. **Evaluation Errors**: Missing variables, invalid paths
4. **Type Errors**: Incompatible type conversions

### Error Propagation

```rust
// Evaluation errors bubble up through the call stack
evaluate_yaml() -> Result<YamlValue>
  ├─ tokenize() -> Result<Vec<Token>>
  ├─ evaluate_provider() -> Result<YamlValue>
  └─ interpolate() -> Result<String>
```

### Error Messages

- **Descriptive**: Include context about what was being evaluated
- **Actionable**: Suggest potential fixes where possible
- **Consistent**: Follow standard format across all providers

## Performance Considerations

### Optimization Strategies

1. **Regex Compilation**: Compile regexes once and reuse
2. **Provider Caching**: Cache provider instances
3. **Token Reuse**: Minimize re-parsing of identical expressions
4. **Lazy Evaluation**: Only evaluate expressions when needed

### Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Simple variable access | O(1) | HashMap lookup |
| Nested path resolution | O(n) | n = path depth |
| JQ expression | O(m) | m = data size |
| String interpolation | O(k) | k = number of tokens |

### Memory Usage

- **Context Tree**: Built once per evaluation session
- **Provider Instances**: Lightweight, can be cached
- **Intermediate Results**: Minimal allocation for most operations

## Extensibility

### Adding New Providers

```rust
struct CustomProvider {
    config: CustomConfig,
}

impl ExpressionProvider for CustomProvider {
    fn evaluate(&self, expression: &str) -> Result<serde_yaml::Value> {
        // Custom evaluation logic
    }
    
    fn name(&self) -> &str {
        "custom"
    }
}

// Register provider
evaluator.register_provider(Box::new(CustomProvider::new(config)));
```

### Provider Registration

Currently, providers are built-in, but the architecture supports dynamic registration:

```rust
// Future extension point
impl ExpressionEvaluator {
    pub fn register_provider(&mut self, provider: Box<dyn ExpressionProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }
}
```

## Security Considerations

### Input Validation

- **Expression Syntax**: Validated during tokenization
- **Provider Names**: Restricted to alphanumeric characters
- **Path Traversal**: Context paths are validated to prevent injection

### Sandboxing

- **JQ Expressions**: Run in controlled environment via xqpath
- **Environment Access**: Limited to pre-defined variable set
- **Context Isolation**: Action outputs are isolated by action ID

### Attack Vectors

1. **Expression Injection**: Mitigated by strict parsing
2. **Resource Exhaustion**: Controlled by expression complexity limits
3. **Data Exfiltration**: Limited by provider scope boundaries

## Testing Strategy

### Unit Tests

- **Provider Functionality**: Each provider tested independently
- **Tokenizer Logic**: All syntax patterns covered
- **Type Preservation**: Single vs. mixed content handling
- **Error Conditions**: All error paths validated

### Integration Tests

- **End-to-End Flow**: Complete evaluation cycles
- **Backward Compatibility**: All legacy syntaxes verified
- **Performance**: Regression testing for common patterns
- **Edge Cases**: Complex nested expressions

### Test Coverage

- **Line Coverage**: >90% for expression module
- **Branch Coverage**: All conditional paths tested
- **Error Coverage**: All error conditions triggered

## Migration Guide

### For Users

1. **Immediate**: Continue using existing syntax (fully supported)
2. **Gradual**: Adopt unified syntax for new workflows
3. **Optional**: Migrate existing workflows when convenient

### For Developers

1. **API Compatibility**: `evaluate()` method unchanged
2. **New Methods**: `evaluate_yaml()` for type preservation
3. **Provider Extension**: Use trait-based approach for new providers

## Future Enhancements

### Planned Features

1. **Dynamic Provider Registration**: Runtime provider addition
2. **Expression Caching**: Cache compiled expressions
3. **Async Providers**: Support for async data sources
4. **Expression Debugging**: Debug information for complex expressions

### Potential Providers

1. **HTTP Provider**: Fetch data from REST APIs
2. **Database Provider**: Query databases directly
3. **File Provider**: Read from local files
4. **Secret Provider**: Integrate with secret management systems

### Performance Improvements

1. **Compiled Expressions**: Pre-compile frequently used expressions
2. **Streaming Evaluation**: Process large data sets efficiently
3. **Parallel Provider Evaluation**: Evaluate independent expressions concurrently

## Conclusion

The provider-based expression system provides a clean, extensible architecture that:

- **Unifies** all expression evaluation under a consistent interface
- **Preserves** backward compatibility with existing syntax
- **Enables** type-safe evaluation with native YAML type preservation
- **Supports** complex data transformation through JQ expressions
- **Provides** clear error handling and debugging capabilities

This design establishes a solid foundation for future enhancements while maintaining the simplicity and power that FlowBuilder users expect.
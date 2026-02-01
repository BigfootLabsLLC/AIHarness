# AIHarness — Agent Coding Guidelines

> **Purpose:** Standards and rules for AI assistants working on this codebase.
>  
> **Applies to:** All AI-generated code contributions.

---

## High-Level Directives

### 1. AI-Developed First

**This is an AI-developed application.** Everything must be buildable by AI without human intervention.

- **Tight, testable loops** — Every component must be verifiable independently
- **Self-sufficient compilation** — I should be able to compile and test without asking you
- **Clear error messages** — If something fails, the error should be actionable
- **Incremental development** — Small, working steps over big bangs

### 2. Architecture Principles

**UI Layer = Thin**
- Minimal logic in React components
- Logic and data live in central libraries (Rust backend)
- UI is a "dumb" view layer
- State management only for UI state, not business logic

**Functional Programming Emphasis**
- Pure functions over stateful classes
- Immutable data structures
- Explicit inputs and outputs
- Avoid side effects; isolate them when necessary

**Test Coverage**
- **Minimum 5 tests per function** (often more)
- Test edge cases, error cases, happy path
- Property-based tests where applicable
- Integration tests for boundaries
- **Test first when possible** (TDD)

---

## Core Philosophy

1. **Clarity over cleverness.** Code should be obvious to a human reading it.
2. **Explicit over implicit.** Types, errors, dependencies — all explicit.
3. **Safety first.** Leverage Rust's type system; no `unwrap()` without comment.
4. **Minimal dependencies.** Prefer stdlib, then well-maintained crates/packages.
5. **The human owns the codebase.** We write what they can maintain.

---

## Stack Overview

| Layer | Technology |
|-------|------------|
| Backend | Rust (Tokio async) |
| Frontend | React + TypeScript |
| Framework | Tauri |
| Database | SQLite (rusqlite) |
| UI Components | Tailwind CSS + Radix UI |
| State | Zustand |
| Diagrams | React Flow |
| Editor | Monaco |

---

## Rust Guidelines

### Functional Programming (Required)

```rust
// DO: Pure functions with explicit inputs/outputs
pub fn calculate_cost(tokens_in: u32, tokens_out: u32, rate: &ModelRate) -> f64 {
    (tokens_in as f64 * rate.input_cost_per_1k / 1000.0) +
    (tokens_out as f64 * rate.output_cost_per_1k / 1000.0)
}

// DON'T: Stateful classes with hidden dependencies
// BAD: impl Calculator { fn new(rate: ModelRate) -> Self; fn calculate(&self, ...) }
// GOOD: Pure function that takes rate as parameter

// DO: Immutable data, return new values
pub fn add_tool(tools: Vec<Tool>, tool: Tool) -> Vec<Tool> {
    let mut new_tools = tools;
    new_tools.push(tool);
    new_tools
}

// DO: Result/Option for error handling, not exceptions
pub fn read_file(path: &Path) -> Result<String, FileError> {
    fs::read_to_string(path)
        .map_err(|e| FileError::ReadFailed(e.to_string()))
}
```

### Code Style

```rust
// DO: Explicit types on public APIs
pub async fn spawn_agent(
    &self,
    config: AgentConfig,
) -> Result<AgentId, AgentError> {
    // ...
}

// DON'T: Implicit returns or inference on boundaries
pub fn get_cost(&self) -> f64 {  // Explicit return type
    self.cost_spent
}
```

### Error Handling

```rust
// DO: ThisError for enum errors, anyhow for application
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("API request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("Rate limit exceeded, retry after {retry_after}s")]
    RateLimited { retry_after: u64 },
    
    #[error("Provider {provider} not configured")]
    ProviderNotConfigured { provider: String },
}

// DO: ? operator with context
let response = client
    .post(&url)
    .json(&request)
    .send()
    .await
    .map_err(|e| LlmError::RequestFailed(e))?;

// DON'T: unwrap() or expect() in production code
// EXCEPTION: Tests and truly invariant conditions (document with comment)
let config = load_config().expect("config must exist at startup"); // OK with comment
```

### Async Patterns

```rust
// DO: Spawn tasks for parallel execution
let handles: Vec<_> = models
    .into_iter()
    .map(|model| {
        tokio::spawn(async move {
            query_model(model).await
        })
    })
    .collect();

// DO: Use join! for bounded parallelism
let (result_a, result_b) = tokio::join!(
    query_model(model_a),
    query_model(model_b),
);

// DON'T: Block the async runtime
// BAD: std::thread::sleep(Duration::from_secs(1));
// GOOD: tokio::time::sleep(Duration::from_secs(1)).await;
```

### Struct Definitions

```rust
// DO: Derive common traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub status: AgentStatus,
    pub created_at: DateTime<Utc>,
}

// DO: Use newtype pattern for IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

// DO: Builder pattern for complex construction
impl Agent {
    pub fn builder() -> AgentBuilder {
        AgentBuilder::default()
    }
}
```

### Module Structure

```rust
// File: src/agents/mod.rs
pub mod manager;
pub mod session;
pub mod types;

pub use manager::AgentManager;
pub use session::AgentSession;
pub use types::{Agent, AgentConfig, AgentId};

// File: src/agents/manager.rs
use crate::agents::{Agent, AgentConfig, AgentId};
use crate::database::Database;

pub struct AgentManager {
    db: Arc<Database>,
    active_sessions: DashMap<AgentId, AgentSession>,
}
```

---

## TypeScript/React Guidelines

### Thin UI Layer (Critical)

**UI components should be "dumb" views. All logic in Rust backend.**

```typescript
// DO: UI calls Rust command, displays result
export function FileViewer({ path }: { path: string }) {
  const [content, setContent] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  
  useEffect(() => {
    setIsLoading(true);
    invoke<string>('read_file', { path })
      .then(setContent)
      .finally(() => setIsLoading(false));
  }, [path]);
  
  if (isLoading) return <Spinner />;
  return <MonacoEditor value={content} readOnly />;
}

// DON'T: Business logic in UI
// BAD: Parsing, calculations, state machines in React
// GOOD: Just display what Rust gives you

// DO: Custom hooks for data fetching, not logic
export function useFileContent(path: string) {
  return useQuery({
    queryKey: ['file', path],
    queryFn: () => invoke<string>('read_file', { path }),
  });
}
```

### Functional Programming (TypeScript)

```typescript
// DO: Pure functions
type CostCalculator = (tokensIn: number, tokensOut: number, rate: ModelRate) => number;

const calculateCost: CostCalculator = (tokensIn, tokensOut, rate) => {
  return (tokensIn * rate.inputCostPer1k / 1000) +
         (tokensOut * rate.outputCostPer1k / 1000);
};

// DO: Immutable updates
const addTool = (tools: Tool[], tool: Tool): Tool[] => [...tools, tool];

// DO: Avoid classes, use functions and types
type Agent = {
  id: string;
  name: string;
  status: AgentStatus;
};

// Not: class Agent { ... }

// DO: Explicit error handling
type Result<T, E = Error> = 
  | { ok: true; value: T }
  | { ok: false; error: E };

const readFile = async (path: string): Promise<Result<string>> => {
  try {
    const content = await invoke<string>('read_file', { path });
    return { ok: true, value: content };
  } catch (error) {
    return { ok: false, error: error as Error };
  }
};
```

### Type Safety

```typescript
// DO: Strict types, no any
interface Agent {
  id: AgentId;
  name: string;
  status: AgentStatus;
  createdAt: Date;
}

type AgentStatus = 'idle' | 'working' | 'error';

// DON'T: any or implicit any
// BAD: function process(data: any) { ... }
// GOOD: function process(data: AgentInput) { ... }
```

### React Components

```typescript
// DO: Functional components with explicit props
interface AgentCardProps {
  agent: Agent;
  onSelect: (id: AgentId) => void;
  isSelected?: boolean;
}

export function AgentCard({ 
  agent, 
  onSelect, 
  isSelected = false 
}: AgentCardProps): JSX.Element {
  return (
    <div 
      className={cn(
        "p-4 rounded-lg border",
        isSelected && "border-blue-500 bg-blue-50",
        !isSelected && "border-gray-200"
      )}
      onClick={() => onSelect(agent.id)}
    >
      <h3 className="font-semibold">{agent.name}</h3>
      <StatusBadge status={agent.status} />
    </div>
  );
}

// DO: Custom hooks for logic
export function useAgent(agentId: AgentId) {
  const [agent, setAgent] = useState<Agent | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  
  useEffect(() => {
    loadAgent(agentId).then(setAgent).finally(() => setIsLoading(false));
  }, [agentId]);
  
  return { agent, isLoading };
}
```

### State Management (Zustand)

```typescript
// DO: Slice pattern for stores
interface AgentStore {
  agents: Agent[];
  selectedId: AgentId | null;
  setAgents: (agents: Agent[]) => void;
  selectAgent: (id: AgentId) => void;
}

export const useAgentStore = create<AgentStore>((set) => ({
  agents: [],
  selectedId: null,
  setAgents: (agents) => set({ agents }),
  selectAgent: (id) => set({ selectedId: id }),
}));

// DO: Selectors to prevent unnecessary re-renders
const agentCount = useAgentStore((state) => state.agents.length);
```

### Tauri IPC

```typescript
// File: src/lib/tauri.ts
import { invoke } from '@tauri-apps/api/core';

// DO: Wrap invoke calls with proper typing
export async function spawnAgent(
  config: AgentConfig
): Promise<Agent> {
  return invoke<Agent>('spawn_agent', { config });
}

// DO: Handle errors at the boundary
try {
  const agent = await spawnAgent(config);
  // ...
} catch (error) {
  // Convert Tauri error to user-friendly message
  toast.error(`Failed to spawn agent: ${error}`);
}
```

---

## Architecture Rules

### Data Flow

```
React UI → Tauri Command → Rust Handler → Database → Response → UI Update
     ↓                                                        ↓
  Zustand store                                          Events (optional)
```

### Crate/Module Boundaries

```
src/
├── main.rs              # Entry point only
├── lib.rs               # Public exports
├── commands.rs          # Tauri command handlers
├── database/            # SQLite abstraction
│   ├── mod.rs
│   ├── migrations/
│   └── repositories/    # Per-entity queries
├── agents/              # Agent system
│   ├── mod.rs
│   ├── manager.rs       # Spawning, lifecycle
│   ├── session.rs       # Individual agent session
│   └── types.rs         # Agent structs
├── tasks/               # Task system
├── costs/               # Cost tracking
├── llm/                 # LLM provider integration
│   ├── mod.rs
│   ├── provider.rs      # Trait definition
│   ├── openai.rs
│   ├── anthropic.rs
│   └── ollama.rs
├── panels/              # Expert panel system
└── utils/               # Shared utilities
```

### Database Pattern

```rust
// Repository pattern per entity
pub struct AgentRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl AgentRepository {
    pub fn create(&self, agent: &Agent) -> Result<AgentId, DbError> {
        // ...
    }
    
    pub fn get_by_id(&self, id: AgentId) -> Result<Option<Agent>, DbError> {
        // ...
    }
    
    pub fn list_by_project(&self, project_id: ProjectId) -> Result<Vec<Agent>, DbError> {
        // ...
    }
}
```

---

## Code Review Checklist

Before submitting code:

- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` passes
- [ ] TypeScript compiles with strict mode (`tsc --noEmit`)
- [ ] No `unwrap()` or `expect()` without explanatory comment
- [ ] Public functions have doc comments
- [ ] Error cases are handled, not logged and ignored
- [ ] No `console.log` in production code (use proper logging)
- [ ] UI updates are optimistic where appropriate
- [ ] Database queries use parameterized statements (SQL injection safe)

---

## Testing Guidelines

### Minimum 5 Tests Per Function

**Every function must have at least 5 tests:**
1. Happy path (normal input)
2. Edge case (empty, zero, max values)
3. Error case (invalid input)
4. Boundary case (at limits)
5. Property-based or fuzz test (if applicable)

Additional tests for:
- Concurrency/thread safety
- Performance characteristics
- Integration with dependencies

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Example: calculate_cost - 6 tests
    
    #[test]
    fn calculate_cost_happy_path() {
        let rate = ModelRate::new(0.01, 0.03); // $0.01 in, $0.03 out per 1k
        let cost = calculate_cost(1000, 1000, &rate);
        assert_eq!(cost, 0.04); // $0.01 + $0.03
    }
    
    #[test]
    fn calculate_cost_zero_tokens() {
        let rate = ModelRate::new(0.01, 0.03);
        let cost = calculate_cost(0, 0, &rate);
        assert_eq!(cost, 0.0);
    }
    
    #[test]
    fn calculate_cost_large_numbers() {
        let rate = ModelRate::new(0.01, 0.03);
        let cost = calculate_cost(1_000_000, 1_000_000, &rate);
        assert_eq!(cost, 40.0); // $10 + $30
    }
    
    #[test]
    fn calculate_cost_fractional_tokens() {
        let rate = ModelRate::new(0.01, 0.03);
        let cost = calculate_cost(1, 1, &rate);
        assert!((cost - 0.00004).abs() < 0.00001);
    }
    
    #[test]
    fn calculate_cost_different_rates() {
        let cheap = ModelRate::new(0.001, 0.002);
        let expensive = ModelRate::new(0.1, 0.3);
        
        assert!(calculate_cost(1000, 1000, &cheap) < 
                calculate_cost(1000, 1000, &expensive));
    }
    
    #[test]
    fn calculate_cost_precision() {
        let rate = ModelRate::new(0.015, 0.045); // Odd rates
        let cost = calculate_cost(333, 666, &rate);
        // Verify precision is maintained
        assert!(cost > 0.0);
        assert!(cost < 1.0);
    }
    
    // Async test example
    #[tokio::test]
    async fn spawn_agent_creates_valid_agent() {
        let manager = AgentManager::new(mock_db()).await;
        let config = AgentConfig::builder()
            .model("test-model")
            .build();
            
        let agent = manager.spawn_agent(config).await.unwrap();
        
        assert_eq!(agent.status, AgentStatus::Idle);
        assert!(!agent.id.is_empty());
        assert!(agent.created_at <= Utc::now());
    }
    
    #[tokio::test]
    async fn spawn_agent_fails_with_invalid_model() {
        let manager = AgentManager::new(mock_db()).await;
        let config = AgentConfig::builder()
            .model("") // Invalid
            .build();
            
        let result = manager.spawn_agent(config).await;
        assert!(result.is_err());
    }
}
```

### Test-Driven Development (Preferred)

```rust
// 1. Write test first
#[test]
fn parse_tool_call_valid_json() {
    let input = r#"{"name": "read_file", "args": {"path": "/tmp/test"}}"#;
    let result = parse_tool_call(input);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name, "read_file");
}

// 2. Implement to make test pass
pub fn parse_tool_call(input: &str) -> Result<ToolCall, ParseError> {
    serde_json::from_str(input)
        .map_err(|e| ParseError::InvalidJson(e.to_string()))
}

// 3. Add more tests for edge cases
#[test]
fn parse_tool_call_invalid_json() { ... }
#[test]
fn parse_tool_call_missing_name() { ... }
#[test]
fn parse_tool_call_empty_args() { ... }
#[test]
fn parse_tool_call_nested_args() { ... }
```

### TypeScript Tests

```typescript
// DO: Test behavior, not implementation
describe('AgentCard', () => {
  it('calls onSelect when clicked', () => {
    const onSelect = vi.fn();
    render(<AgentCard agent={mockAgent} onSelect={onSelect} />);
    
    fireEvent.click(screen.getByText(mockAgent.name));
    
    expect(onSelect).toHaveBeenCalledWith(mockAgent.id);
  });
});
```

---

## Documentation Standards

### Rust Doc Comments

```rust
/// Spawns a new agent with the given configuration.
///
/// # Arguments
///
/// * `config` - The configuration for the agent, including model and role.
///
/// # Returns
///
/// Returns the ID of the newly spawned agent on success.
///
/// # Errors
///
/// Returns `AgentError::InvalidConfig` if the configuration is incomplete.
/// Returns `AgentError::ProviderNotConfigured` if the LLM provider is not set up.
///
/// # Examples
///
/// ```
/// let config = AgentConfig::builder()
///     .model("claude-3-sonnet")
///     .role(AgentRole::Architect)
///     .build()?;
///     
/// let agent_id = manager.spawn_agent(config).await?;
/// ```
pub async fn spawn_agent(&self, config: AgentConfig) -> Result<AgentId, AgentError> {
    // ...
}
```

### TypeScript TSDoc

```typescript
/**
 * Spawns a new agent with the given configuration.
 * @param config - The configuration for the agent
 * @returns The created agent
 * @throws Will throw if the configuration is invalid
 * 
 * @example
 * ```typescript
 * const agent = await spawnAgent({
 *   model: 'claude-3-sonnet',
 *   role: 'architect'
 * });
 * ```
 */
export async function spawnAgent(config: AgentConfig): Promise<Agent> {
  return invoke('spawn_agent', { config });
}
```

---

## Performance Guidelines

### Rust

- Use `Arc<str>` or `Arc<String>` for shared immutable strings
- Use `DashMap` instead of `Mutex<HashMap>` for concurrent access
- Stream large responses, don't buffer entirely
- Use `tokio::sync::mpsc` for backpressure-aware channels

### Frontend

- Virtualize long lists (`react-window` or `@tanstack/react-virtual`)
- Debounce rapid user input (search, resizes)
- Memoize expensive computations (`useMemo`)
- Lazy load heavy components (`React.lazy`)

---

## Security Rules

- **NEVER** log API keys or tokens
- **NEVER** execute user-provided code without sandboxing
- **ALWAYS** validate input at API boundaries
- **ALWAYS** use parameterized SQL queries
- **ALWAYS** escape HTML when rendering user content (unless using React properly)

---

## Git Conventions

### Commit Messages

```
feat: Add expert panel parallel query execution

Implements concurrent querying of multiple LLM providers
with proper error isolation and cost aggregation.

- Add tokio::spawn for parallel execution
- Implement per-model timeout handling
- Aggregate costs and responses

Fixes #42
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`

### Branch Naming

- `feature/expert-panel-system`
- `fix/agent-memory-leak`
- `refactor/cost-tracker`

---

## Adding to This Document

When new patterns emerge:

1. Add to the relevant section
2. Include concrete code examples (DO/DON'T)
3. Explain the "why" briefly
4. Mark with date and author: `// Added: 2026-02-15`

---

## Questions?

When in doubt:
1. Check existing code for patterns
2. Prefer explicit over clever
3. Ask the human
4. Document the decision

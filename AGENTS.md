# AIHarness — Agent Coding Guidelines

> **Purpose:** Standards and rules for AI assistants working on this codebase.
>  
> **Applies to:** All AI-generated code contributions.

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

### Rust Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // DO: Unit test pure functions
    #[test]
    fn test_calculate_cost() {
        let result = calculate_cost(1000, 500, &MODEL_GPT4);
        assert_eq!(result, 0.045); // $0.045 per call
    }
    
    // DO: Async tests with tokio::test
    #[tokio::test]
    async fn test_spawn_agent() {
        let manager = AgentManager::new(mock_db()).await;
        let agent = manager.spawn_agent(test_config()).await.unwrap();
        assert_eq!(agent.status, AgentStatus::Idle);
    }
}
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

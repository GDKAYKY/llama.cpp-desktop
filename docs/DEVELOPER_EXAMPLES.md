# Developer Examples

Practical examples for common development tasks.

## Backend Examples

### 1. Adding a New Tauri Command

```rust
// 1. Define in commands/my_feature.rs
#[tauri::command]
pub async fn my_command(
    state: tauri::State<'_, AppState>,
    param: String
) -> Result<String, String> {
    // Access services from state
    let service = &state.my_service;
    
    // Call service method
    service.do_something(param)
        .await
        .map_err(|e| e.to_string())
}

// 2. Register in ipc_handlers.rs
pub fn configure_ipc(builder: tauri::Builder) -> tauri::Builder {
    builder.invoke_handler(tauri::generate_handler![
        // ... existing commands
        commands::my_feature::my_command,
    ])
}

// 3. Call from frontend
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('my_command', { param: 'value' });
```

### 2. Creating a New Service

```rust
// services/my_service.rs
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MyService {
    state: Arc<Mutex<MyState>>,
}

impl MyService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MyState::default())),
        }
    }
    
    pub async fn do_something(&self, input: String) -> Result<String, String> {
        let mut state = self.state.lock().await;
        // Business logic here
        Ok(format!("Processed: {}", input))
    }
}

// Add to state.rs
pub struct AppState {
    // ... existing fields
    pub my_service: Arc<MyService>,
}

impl AppState {
    pub fn new(/* params */) -> Self {
        Self {
            // ... existing fields
            my_service: Arc::new(MyService::new()),
        }
    }
}
```

### 3. Adding a New Model

```rust
// models/my_model.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyModel {
    pub id: String,
    pub name: String,
    pub data: Vec<String>,
}

impl Default for MyModel {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            data: Vec::new(),
        }
    }
}

// Add to lib.rs models module
pub mod models {
    // ... existing models
    pub mod my_model;
    pub use my_model::*;
}
```

### 4. Using the Actor Pattern

```rust
// Define messages
pub enum MyActorMessage {
    DoWork(String),
    GetState,
}

// Actor implementation
pub struct MyActor {
    state: MyState,
}

impl MyActor {
    async fn handle_message(&mut self, msg: MyActorMessage) -> Result<String, String> {
        match msg {
            MyActorMessage::DoWork(input) => {
                self.state.process(input);
                Ok("Done".to_string())
            }
            MyActorMessage::GetState => {
                Ok(format!("{:?}", self.state))
            }
        }
    }
    
    pub fn spawn() -> mpsc::Sender<MyActorMessage> {
        let (tx, mut rx) = mpsc::channel(32);
        
        tokio::spawn(async move {
            let mut actor = MyActor { state: MyState::default() };
            
            while let Some(msg) = rx.recv().await {
                let _ = actor.handle_message(msg).await;
            }
        });
        
        tx
    }
}
```

## Frontend Examples

### 1. Creating a New Svelte Store

```typescript
// src/lib/stores/my-store.svelte.ts
import { invoke } from '@tauri-apps/api/core';

interface MyState {
    data: string[];
    loading: boolean;
    error: string | null;
}

let state = $state<MyState>({
    data: [],
    loading: false,
    error: null
});

export const myStore = {
    get data() { return state.data; },
    get loading() { return state.loading; },
    get error() { return state.error; },
    
    async loadData() {
        state.loading = true;
        state.error = null;
        
        try {
            const result = await invoke<string[]>('get_my_data');
            state.data = result;
        } catch (err) {
            state.error = err as string;
        } finally {
            state.loading = false;
        }
    },
    
    addItem(item: string) {
        state.data = [...state.data, item];
    }
};
```

### 2. Creating a New Component

```svelte
<!-- src/components/MyComponent.svelte -->
<script lang="ts">
    import { myStore } from '$lib/stores/my-store.svelte';
    import { onMount } from 'svelte';
    
    let { title = 'Default Title' } = $props();
    
    onMount(() => {
        myStore.loadData();
    });
    
    function handleClick() {
        myStore.addItem('New item');
    }
</script>

<div class="my-component">
    <h2>{title}</h2>
    
    {#if myStore.loading}
        <p>Loading...</p>
    {:else if myStore.error}
        <p class="error">{myStore.error}</p>
    {:else}
        <ul>
            {#each myStore.data as item}
                <li>{item}</li>
            {/each}
        </ul>
    {/if}
    
    <button onclick={handleClick}>Add Item</button>
</div>

<style>
    .my-component {
        padding: 1rem;
    }
    
    .error {
        color: red;
    }
</style>
```

### 3. Using IPC with Streaming

```typescript
// Listen to streaming events
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<string>('my-stream-event', (event) => {
    console.log('Received:', event.payload);
    // Update UI with streamed data
});

// Don't forget to unlisten when component unmounts
onDestroy(() => {
    unlisten();
});
```

### 4. IndexedDB Operations

```typescript
// src/lib/services/my-db.ts
import Dexie, { type Table } from 'dexie';

interface MyRecord {
    id?: number;
    name: string;
    data: string;
    timestamp: number;
}

class MyDatabase extends Dexie {
    records!: Table<MyRecord>;
    
    constructor() {
        super('MyDatabase');
        this.version(1).stores({
            records: '++id, name, timestamp'
        });
    }
}

const db = new MyDatabase();

export async function saveRecord(record: Omit<MyRecord, 'id'>) {
    await db.records.add(record);
}

export async function getRecords() {
    return await db.records.toArray();
}

export async function searchRecords(query: string) {
    return await db.records
        .where('name')
        .startsWithIgnoreCase(query)
        .toArray();
}
```

## Testing Examples

### Backend Unit Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_my_service() {
        let service = MyService::new();
        let result = service.do_something("test".to_string()).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Processed: test");
    }
}
```

### Frontend Unit Test

```typescript
// tests/stores/my-store.test.ts
import { describe, it, expect, vi } from 'vitest';
import { myStore } from '$lib/stores/my-store.svelte';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn().mockResolvedValue(['item1', 'item2'])
}));

describe('myStore', () => {
    it('loads data successfully', async () => {
        await myStore.loadData();
        
        expect(myStore.data).toEqual(['item1', 'item2']);
        expect(myStore.loading).toBe(false);
        expect(myStore.error).toBe(null);
    });
    
    it('adds item to data', () => {
        myStore.addItem('item3');
        
        expect(myStore.data).toContain('item3');
    });
});
```

## Common Patterns

### Error Handling

```rust
// Backend
pub async fn my_command() -> Result<String, String> {
    some_operation()
        .await
        .map_err(|e| format!("Failed to do something: {}", e))
}
```

```typescript
// Frontend
try {
    const result = await invoke('my_command');
    // Success
} catch (error) {
    console.error('Error:', error);
    // Show error to user
}
```

### Configuration Management

```rust
// Backend - Read config
let config = commands::config::get_config(app.handle())?;

// Backend - Save config
commands::config::save_config(app.handle(), new_config)?;
```

```typescript
// Frontend
import { invoke } from '@tauri-apps/api/core';

const config = await invoke('get_config');
await invoke('save_config', { config: newConfig });
```

---

*Last updated: 2026-03-28*

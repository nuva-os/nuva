# Nuva Programming Language Reference

> Version 1.0 — May 2026

## 1. Overview

Nuva is a declarative programming language designed for building UI-centric applications on Nuva OS. It emphasizes:

- **Declarative paradigm** — describe *what*, not *how*
- **Reactive data flow** — automatic change propagation via `signal`/`effect`
- **Declarative concurrency** — `async`/`await` compiled to state machines
- **Declarative resource management** — `resource`/`with` for RAII-style cleanup
- **Pipeline and comprehension syntax** — functional data transformation
- **Type safety** — static type checking with inference

Nuva source files use the `.nv` extension.

---

## 2. Design Philosophy

### 2.1 Declarative over Imperative

Nuva replaces imperative patterns with declarative equivalents:

| Imperative Pattern | Nuva Declarative Equivalent |
|--------------------|----------------------------|
| `new Widget()` + `build()` | `component` declaration |
| `setState()` + `notify()` | `signal` with automatic propagation |
| `addEventListener()` | Modifier-bound `.on_click()` |
| `Thread.start()` + callbacks | `async`/`await` |
| `try { acquire() } finally { release() }` | `resource`/`with` |
| `.then().then()` promise chain | `await` chain |

### 2.2 Zero-Cost Abstractions

Declarative constructs compile to efficient runtime representations:

- `component` → reconciled element tree (O(n) diff)
- `signal` → atomic version counter + dirty marking
- `effect` → dependency graph + topological scheduling
- `async` → state machine IR (no heap allocation for simple cases)
- `resource`/`with` → scoped acquire/release with guaranteed cleanup

### 2.3 Safety

- Static type checking and inference
- Purity verification for effect bodies
- Declarative constraint validation at compile time
- Guaranteed resource cleanup (no leaks)

---

## 3. Keywords Reference

### 3.1 Core Keywords

| Keyword | Category | Description |
|---------|----------|-------------|
| `fn` | Declaration | Function declaration |
| `let` | Binding | Immutable binding |
| `var` | Binding | Mutable binding |
| `const` | Binding | Compile-time constant |
| `type` | Declaration | Type alias |
| `struct` | Declaration | Structure type |
| `enum` | Declaration | Enumeration type |
| `trait` | Declaration | Trait (interface) |
| `impl` | Declaration | Implementation |
| `if` | Control | Conditional branch |
| `else` | Control | Alternative branch |
| `match` | Control | Pattern matching |
| `loop` | Control | Infinite loop |
| `while` | Control | Conditional loop |
| `for` | Control | Iterator loop |
| `break` | Control | Loop exit |
| `continue` | Control | Loop skip |
| `return` | Control | Function return |
| `true` | Literal | Boolean true |
| `false` | Literal | Boolean false |
| `self` | Binding | Self reference |
| `super` | Binding | Parent scope |
| `use` | Module | Import |
| `mod` | Module | Module declaration |
| `pub` | Visibility | Public visibility |
| `as` | Conversion | Type cast / alias |

### 3.2 Declarative Keywords

| Keyword | Category | Description |
|---------|----------|-------------|
| `component` | Declarative | Declare a UI component |
| `signal` | Declarative | Declare a reactive state variable |
| `effect` | Declarative | Register a reactive side effect |
| `reactive` | Declarative | Mark a function as reactive-safe |
| `async` | Concurrency | Mark a function as asynchronous |
| `await` | Concurrency | Suspend until Future resolves |
| `resource` | Resource | Declare a resource type with acquire/release |
| `with` | Resource | Scoped resource binding with auto-cleanup |

### 3.3 Pipeline Keywords

| Keyword | Category | Description |
|---------|----------|-------------|
| `pipeline` | Pipeline | Declare a data processing pipeline |
| `yield` | Pipeline | Emit a value in a pipeline/generator |
| `filter` | Comprehension | Filter clause in comprehension |
| `map` | Comprehension | Transform clause in comprehension |

---

## 4. Type System

### 4.1 Primitive Types

| Type | Description | Size |
|------|-------------|------|
| `Int` | Signed integer | Platform-dependent (32/64-bit) |
| `Int8` | Signed 8-bit integer | 1 byte |
| `Int16` | Signed 16-bit integer | 2 bytes |
| `Int32` | Signed 32-bit integer | 4 bytes |
| `Int64` | Signed 64-bit integer | 8 bytes |
| `UInt` | Unsigned integer | Platform-dependent |
| `UInt8` | Unsigned 8-bit integer | 1 byte |
| `UInt16` | Unsigned 16-bit integer | 2 bytes |
| `UInt32` | Unsigned 32-bit integer | 4 bytes |
| `UInt64` | Unsigned 64-bit integer | 8 bytes |
| `Float` | Floating-point | 8 bytes (f64) |
| `Float32` | Single-precision float | 4 bytes |
| `Bool` | Boolean | 1 byte |
| `Char` | Unicode character | 4 bytes |
| `String` | UTF-8 string | Variable |
| `Unit` | Unit type `()` | 0 bytes |

### 4.2 Special Types

| Type | Description |
|------|-------------|
| `Reactive<T>` | Reactive wrapper — reads trigger dependency tracking, writes propagate to effects |
| `Future<T>` | Asynchronous computation result — resolved with `await` |
| `Resource<T>` | Managed resource — guaranteed acquire/release lifecycle |
| `Result<T, E>` | Success (`Ok(T)`) or error (`Err(E)`) |
| `Option<T>` | Present (`Some(T)`) or absent (`None`) |

### 4.3 Collection Types

| Type | Description |
|------|-------------|
| `Vec<T>` | Dynamic array |
| `HashMap<K, V>` | Hash map (SipHash, chaining, auto-rehash) |
| `HashSet<T>` | Hash set |
| `LinkedList<T>` | Doubly-linked list |
| `String` | UTF-8 string (growable) |

### 4.4 Type Inference

Nuva supports type inference with explicit annotation where needed:

```nuva
let x = 42              // inferred as Int
let y: Float = 3.14     // explicit Float annotation
let z = x + y           // inferred as Float (implicit conversion)

signal count: Int = 0   // explicit type required for signals
```

---

## 5. Declarative UI Paradigm

### 5.1 Component Declaration

```nuva
component Greeting(name: String) {
    Column {
        Text("Hello, " + name)
            .font_size(24)
            .font_weight(Bold)
            .padding(16)
    }
}
```

### 5.2 Component Composition

```nuva
component App() {
    Column {
        Header(title: "My App")
        Content()
        Footer()
    }
}
```

### 5.3 Conditional and Loop Rendering

```nuva
component TodoList(items: Vec<Todo>) {
    Column {
        if items.is_empty() {
            Text("No items")
        } else {
            for item in items {
                TodoRow(todo: item)
            }
        }
    }
}
```

### 5.4 Modifier Chains

Modifiers are chainable and apply layout, styling, events, and accessibility:

```nuva
Text("Submit")
    .font_size(16)
    .font_color(Color.Blue)
    .padding(8, 16)
    .background(Color.White)
    .border_radius(4)
    .on_click(handle_submit)
```

---

## 6. Reactive Binding

### 6.1 Signal Declaration

```nuva
signal username: String = ""
signal is_logged_in: Bool = false
```

### 6.2 Signal Mutation

```nuva
fn on_login(name: String) {
    username = name
    is_logged_in = true
}
```

### 6.3 Effect Registration

```nuva
effect {
    // Runs initially, then re-runs whenever `username` changes
    console.log("User: " + username)
}

effect {
    if is_logged_in {
        fetch_profile(username)
    }
}
```

### 6.4 Reactive Computation

```nuva
signal first_name: String = "Alice"
signal last_name: String = "Smith"

effect {
    // Computed reactively from dependencies
    let full_name = first_name + " " + last_name
    update_display(full_name)
}
```

### 6.5 Reactivity Rules

1. **Signal reads are tracked** — any `signal` read inside an `effect` body creates a dependency
2. **Writes propagate** — modifying a `signal` triggers all dependent effects
3. **Effects are scheduled** — multiple effects are batched and run in topological order
4. **No infinite loops** — the scheduler detects and prevents circular dependencies

---

## 7. Declarative Concurrency

### 7.1 Async Functions

```nuva
async fn load_user(id: Int) -> Result<User, Error> {
    let response = await http.get("/api/users/" + id.to_string())
    let user = await response.json()
    return user
}
```

### 7.2 Concurrent Composition

```nuva
async fn load_dashboard() -> Dashboard {
    let user = load_user(current_user_id)
    let posts = load_posts()
    let stats = load_stats()

    // All three requests run concurrently
    return Dashboard(
        user: await user,
        posts: await posts,
        stats: await stats
    )
}
```

### 7.3 Error Handling in Async

```nuva
async fn safe_fetch(url: String) -> Result<Data, Error> {
    match await http.get(url) {
        Ok(response) if response.status == 200 => Ok(await response.json()),
        Ok(response) => Err(Error.HttpError(response.status)),
        Err(e) => Err(e),
    }
}
```

### 7.4 Compilation to State Machine

The Nuva compiler transforms `async` functions into state machine IR:

1. Each `await` point becomes a suspension point
2. Local variables are captured in the state machine struct
3. Resumption continues from the next state
4. No heap allocation for simple single-`await` functions

---

## 8. Declarative Resource Management

### 8.1 Resource Declaration

```nuva
resource DatabaseConnection(config: DbConfig) {
    acquire: db.connect(config),
    release: conn.close()
}
```

### 8.2 Scoped Usage with `with`

```nuva
with (conn = DatabaseConnection(default_config)) {
    let result = conn.query("SELECT * FROM users")
    process(result)
}
// conn.close() is called automatically here
```

### 8.3 Nested Resources

```nuva
with (conn = DatabaseConnection(config)) {
    with (tx = conn.begin_transaction()) {
        tx.execute("INSERT INTO users ...")
        tx.execute("UPDATE counters ...")
        tx.commit()
    }
    // tx is rolled back if commit was not called
}
// conn is closed
```

### 8.4 Resource Guarantees

1. **Acquire** — the `acquire` expression runs when entering the `with` scope
2. **Release** — the `release` expression runs when exiting the `with` scope, even on exception
3. **No leak** — resources cannot be moved outside their `with` scope
4. **Ordering** — nested resources are released in reverse order (LIFO)

---

## 9. Pipeline and Comprehension Syntax

### 9.1 Pipeline Operator

```nuva
let result = data
    |> filter(|x| x > 0)
    |> map(|x| x * 2)
    |> reduce(0, |acc, x| acc + x)
```

### 9.2 Comprehension Syntax

```nuva
let squares = [x * x for x in 0..10 if x % 2 == 0]
// [0, 4, 16, 36, 64]
```

### 9.3 Pipeline Declaration

```nuva
pipeline EtlPipeline {
    stage Extract {
        yield read_csv("input.csv")
    }

    stage Transform {
        yield row
            |> normalize()
            |> validate()
    }

    stage Load {
        write_db(row)
    }
}
```

### 9.4 Pipeline IR

The compiler generates specialized IR for pipelines:

- Each `stage` becomes a separate IR node
- `yield` creates suspend/resume points
- Data flows between stages without intermediate collections (streaming)
- Comprehensions are optimized to avoid allocations when possible

---

## 10. Operator Reference

### 10.1 Arithmetic Operators

| Operator | Name | Precedence | Associativity |
|----------|------|------------|---------------|
| `^` | Exponentiation | 7 (highest) | Right |
| `*` | Multiplication | 6 | Left |
| `/` | Division | 6 | Left |
| `%` | Modulo | 6 | Left |
| `+` | Addition | 5 | Left |
| `-` | Subtraction | 5 | Left |

### 10.2 Comparison Operators

| Operator | Name | Precedence |
|----------|------|------------|
| `==` | Equal | 4 |
| `!=` | Not equal | 4 |
| `<` | Less than | 4 |
| `>` | Greater than | 4 |
| `<=` | Less or equal | 4 |
| `>=` | Greater or equal | 4 |

### 10.3 Logical Operators

| Operator | Name | Precedence | Associativity |
|----------|------|------------|---------------|
| `not` | Logical NOT | 3 (prefix) | — |
| `and` | Logical AND | 2 | Left |
| `or` | Logical OR | 1 (lowest) | Left |

### 10.4 Pipeline Operator

| Operator | Name | Precedence |
|----------|------|------------|
| `\|>` | Pipeline | 0 (lowest) |

---

## 11. File Format

### 11.1 File Extension

All Nuva source files use the `.nv` extension:

```
my_app/
├── main.nv
├── components/
│   ├── header.nv
│   ├── footer.nv
│   └── sidebar.nv
├── services/
│   └── api.nv
└── styles/
    └── theme.nv
```

### 11.2 Module System

```nuva
// Import from another module
use components.header.Header
use services.api.{fetch_user, fetch_posts}

// Re-export
pub use components.*
```

### 11.3 Entry Point

A Nuva application's entry point is `main.nv` with a `main` function:

```nuva
fn main() {
    App().render()
}
```

---

## 12. Compiler Pipeline

### 12.1 Stages

```
.nv Source
    │
    ▼
┌─────────┐
│  Lexer   │  Tokenization with multi-radix numbers, declarative keywords
└─────────┘
    │
    ▼
┌─────────┐
│  Parser  │  Pratt priority parsing, declarative syntax (component/signal/effect/async/resource/with)
└─────────┘
    │
    ▼
┌──────────────┐
│  Semantic    │  Type checking, type inference, purity verification, declarative constraints
│  Analysis    │
└──────────────┘
    │
    ▼
┌──────────────┐
│  Code Gen    │  Pipeline IR, comprehension IR, async state machine IR, reactive IR
└──────────────┘
    │
    ▼
┌──────────────┐
│  IR Optimizer│  Constant folding, DCE, CSE, copy propagation, loop optimization, inlining
└──────────────┘
    │
    ▼
┌──────────────┐
│  Backend     │  VM bytecode or native code generation (NEX format)
└──────────────┘
```

### 12.2 Runtime

| Component | Description |
|-----------|-------------|
| VM | 256-register virtual machine with instruction dispatch |
| GC | Mark-sweep garbage collection with root scanning |
| Reactive Scheduler | Dependency graph, topological sort, effect batching |
| NEX Loader | Binary module loading with relocation |

---

## 13. Standard Library

### 13.1 Core

- `Vec<T>`, `String`, `HashMap<K, V>`, `LinkedList<T>`
- `Option<T>`, `Result<T, E>`
- `Int`, `Float`, `Bool`, `Char`

### 13.2 IO

- `Stdin`, `Stdout`, `Stderr`
- `File` (read, write, open, close)
- `Path`

### 13.3 Math

- Trigonometric: `sin`, `cos`, `tan`
- Exponential: `exp`, `log`, `log2`, `log10`
- Power: `pow`, `sqrt`
- Rounding: `floor`, `ceil`, `round`

### 13.4 Reactive

- `signal`, `effect`, `reactive`
- `Reactive<T>` type

### 13.5 Async

- `Future<T>`, `spawn`, `await`
- `Channel<T>` for async message passing

### 13.6 Resource

- `Resource<T>`, `with`
- Built-in resources: `FileHandle`, `DatabaseConnection`, `NetworkSocket`

### 13.7 Collections

- `Vec<T>`: `push`, `pop`, `iter`, `map`, `filter`, `reduce`
- `HashMap<K, V>`: `insert`, `get`, `remove`, `contains`, `iter`
- `String`: `concat`, `split`, `trim`, `contains`, `to_upper`, `to_lower`

---

**Last Updated**: May 30, 2026

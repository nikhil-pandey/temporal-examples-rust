# Activities Dependency Injection Example

Demonstrates dependency injection for activities, mirroring the [TypeScript activities-dependency-injection sample](https://github.com/temporalio/samples-typescript/tree/main/activities-dependency-injection).

## Pattern

- **Greeter Service**: A `Greeter` struct is created for each language, with a `greet(name)` method.
- **Activities**: Each activity accesses its dependency (the Greeter) by reading a static reference initialized by the worker. This enables safe sharing (dependency injection pattern) across all activity invocations at runtime.
- **Workflow**: Invokes both activities (English & Spanish) and returns their results as a concatenated string.

## How to run

1. Start Temporal server:
   ```
   temporal server start-dev
   ```
2. In this directory, start the worker:
   ```
   cargo run --bin worker
   ```
3. In another terminal, start the client:
   ```
   cargo run --bin client -- Alice
   ```
   Optionally, specify a name to greet (defaults to "World").

### Output
Should show two greetings (English & Spanish):

```
Hello, Alice!
Hola, Alice!
```

## Key points
- Activities use dependency injection by having their dependencies set at worker startup as statics.
- No global mutable state is exposed; only `Arc<Greeter>` statics via `once_cell::sync::Lazy`.
- Pattern matches the core intent of the TypeScript sample for sharing expensive resources (DB pools, HTTP clients, etc) safely per worker process.
- This is preferred to singletons, and is closer to production service patterns.

## Mapping to TypeScript sample
- TS [sample here](https://github.com/temporalio/samples-typescript/tree/main/activities-dependency-injection): Each activity receives a dependency.
- Rust: use Lazy statics for similar effect, as passing state to activities directly is not yet SDK-supported.

See root [README](../../README.md) for workspace and more examples.


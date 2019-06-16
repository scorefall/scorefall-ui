# scorefall-ui
User interface for ScoreFall™.

## Web Interface
### Installing
```bash
cargo install cargo-web --force
```

### Building
```
cargo web start --target=wasm32-unknown-unknown --release
```

### Releasing
```
cargo web deploy --target=wasm32-unknown-unknown --release
```

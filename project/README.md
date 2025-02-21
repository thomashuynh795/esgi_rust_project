## DEVELOPMENT

```cargo watch -x "build --bin worker" -x test```

```../server_aarch64_apple_darwin --debug run```

```./target/debug/worker localhost:8778```

## PRODUCTION

```cargo build --release```

```../server_aarch64_apple_darwin run```

```./target/release/worker localhost:8778```

# json2kvd

Convert JSON documents to KVD — and KVD to JSON with `--reverse`.

KVD is a line-oriented config/data format that keeps YAML/JSON readability without pitfalls: strict 2-space indentation, no flow collections, no implicit coercion, exactly one way to spell most things.

## Install

```sh
cargo install json2kvd
```

## Usage

```sh
json2kvd values.json > values.kvd          # optional: --schema schema.json
json2kvd --reverse values.kvd > values.json
```

- Without `--reverse`, reads JSON and writes KVD to stdout.
- With `--schema <schema.json>`, the JSON schema is converted to a KVD schema node and the converted document is verified against it before emitting.
- With `--reverse <input.kvd>`, reads KVD and writes JSON (pretty-printed) to stdout. `--schema` is not valid with `--reverse`.

## Example

Input `values.json`:

```json
{
  "app": {
    "name": "hello",
    "port": 8080
  }
}
```

Output `values.kvd`:

```
app:
  name: "hello"
  port: 8080
```

Reverse:

```sh
json2kvd --reverse values.kvd
```

## Library

```toml
[dependencies]
json2kvd = "1.0.0"
kvd-rs = "1.0.0"
serde_json = "1.0.151"
```

```rust
let kvd = json2kvd::json_text_to_kvd(r#"{"port": 8080}"#, None)?;
let json = json2kvd::kvd_text_to_json("port: 8080\n")?;
```

Core parsing/serialization and schema verification live in [`kvd-rs`](https://crates.io/crates/kvd-rs).

## License

MIT

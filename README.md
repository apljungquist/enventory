# enventory

_A typed, self-documenting environment variable registry for Rust._

**For library authors**:
- Interact with strongly typed constants instead of stringly typed environment variables

**For binary authors**:
- Avoid surprising behavior due to misconfigured environment variable
- See what environment variables affect the program in the help text

## Quick start

### Declaring variables (library crate)

Add `enventory` as a dependency (no features needed):

```rust
enventory::define! {
    /// Port to listen on
    pub static MY_PORT: u16 = 8080
}

pub fn start_server() {
    let port = *MY_PORT;
    println!("Listening on port {port}");
}
```

`MY_PORT` will always be a `u16`.
If the environment variable is not set or cannot be parsed,
then the fallback value of `8080` is used.

If feature unification does not enable the `inventory` feature then this expands to roughly:

```rust
static MY_PORT: Var<u16, <u16 as FromStr>::Err> = Var::new("MY_PORT", <u16 as FromStr>::from_str, || 8080);
```

Which is roughly equivalent to:

```rust
static MY_PORT: LazyLock<u16> = LazyLock::new(|| {
    env::var("MY_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080)
});
```

See [example-lib](examples/example-lib/src/lib.rs) for more patterns.

### Validating (binary crate)

Enable the `clap` feature and flatten [`EnvArgs`] into your CLI struct:

```rust,ignore
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    env: enventory::EnvArgs,
}

fn main() {
    let _cli = Cli::parse();
    serve();
}
```

If `MYHTTP_PORT` is set and cannot be parsed,
then the error will be reported to the user when the arguments are parsed.

Running the program with `--help` prints:

```text
Usage: example-minimal [OPTIONS]

Options:
      --my-port <MY_PORT>  Port to listen on [env: MY_PORT=] [default: 8080]
  -h, --help               Print help
```

Other patterns:
- [procedural clap](examples/example-bin-clap/src/procedural.rs)
- [without clap](examples/example-bin-inventory/src/main.rs)
- [without validation](examples/example-bin-no-inventory/src/main.rs)

## Cargo features

| Feature     | Default | Description                                                                                                            |
|-------------|---------|------------------------------------------------------------------------------------------------------------------------|
| `inventory` | no      | Enables [`validate_all()`], error types, and the `inventory` registry. Intended for binary crates that don't use clap. |
| `clap`      | no      | Implies `inventory`. Enables [`EnvArgs`], [`args()`], and [`apply_matches()`]. Adds a dependency on `clap`.            |

## Related projects

- [envy](https://crates.io/crates/envy):
  - Deserializes environment variables into a struct via serde.
  - Facilitates parsing, but not caching or discoverability by users.
- [figment](https://crates.io/crates/figment):
  - A layered configuration framework that merges files, environment variables, CLI args, and other sources.
  - Facilitates parsing and merging sources but not caching or discoverability by users.
- [envconfig](https://crates.io/crates/envconfig):
  - Derive macro that populates a struct from environment variables.
  - Facilitates parsing, but not caching or discoverability by users.
- [dotenvy](https://crates.io/crates/dotenvy):
  - Loads `.env` files into the process environment.
  - Complementary; it sets variables that enventory (or any other reader) can then parse.
- [clap](https://crates.io/crates/clap):
  - Parses command line arguments and, with the `env` feature, environment variable into a struct.
  - Facilitates parsing and discoverability by users, but only for environment variables known to the binary.

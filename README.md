# Duration Flex

Helper class to make it easier to specify duration files. Specially useful in configuration files.

**Example:**
- 1 hour and 23 minutes can be written as: `1h23m`

## Developing

1. Install fish shell.
2. Install a recent (1.80+) rust compiler (with cargo).
3. Install a toolchain compatible with the desired target, like `stable-aarch64-apple-darwin`.
```shell
rustup toolchain install stable-aarch64-apple-darwin
```
4. Install a nightly profile compatible with the current machine, like:
```shell
rustup toolchain install nightly-aarch64-apple-darwin
```
5. Install rust packages:
```shell
cargo install --profile release taplo ripgrep
```
6. Setup hooks: `./scripts/setup-hooks`
7. (**Optional**) Run `./scripts/setup-target` to initialize `target/` in the temporary directory. This has to be done everytime the machine is restarted. 

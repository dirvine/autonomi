# `autonomi` - Autonomi client API

[![Crates.io](https://img.shields.io/crates/v/autonomi.svg)](https://crates.io/crates/autonomi)
[![docs.rs](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/autonomi)

Connect to and build on the Autonomi network.

## Usage

Add the autonomi crate to your `Cargo.toml`:

```toml
[dependencies]
autonomi = { path = "../autonomi", version = "0.1.0" }
```

## Running tests

### Using a local EVM testnet

1. If you haven't, install Foundry, to be able to run Anvil
   nodes: https://book.getfoundry.sh/getting-started/installation
2. Run a local EVM node:

```sh
cargo run --bin evm_testnet
```

3. Run a local network with the `local` feature and use the local evm node.

```sh
cargo run --bin=safenode-manager --features=local -- local run --build --clean --rewards-address <ETHEREUM_ADDRESS> evm-local
```

4. Then run the tests with the `local` feature and pass the EVM params again:

```sh
EVM_NETWORK=local cargo test --package=autonomi --features=local
# Or with logs
RUST_LOG=autonomi EVM_NETWORK=local cargo test --package=autonomi --features=local -- --nocapture
```

### Using a live testnet or mainnet

Using the hardcoded `Arbitrum One` option as an example, but you can also use the command flags of the steps above and
point it to a live network.

1. Run a local network with the `local` feature:

```sh
cargo run --bin=safenode-manager --features=local -- local run --build --clean --rewards-address <ETHEREUM_ADDRESS> evm-arbitrum-one
```

2. Then run the tests with the `local` feature. Make sure that the wallet of the private key you pass has enough gas and
   payment tokens on the network (in this case Arbitrum One):

```sh
EVM_NETWORK=arbitrum-one EVM_PRIVATE_KEY=<PRIVATE_KEY> cargo test --package=autonomi --features=local
# Or with logs
RUST_LOG=autonomi EVM_NETWORK=arbitrum-one EVM_PRIVATE_KEY=<PRIVATE_KEY> cargo test --package=autonomi --features=local -- --nocapture
```

### WebAssembly

To run a WASM test

- Install `wasm-pack`
- Make sure your Rust supports the `wasm32-unknown-unknown` target. (If you
  have `rustup`: `rustup target add wasm32-unknown-unknown`.)
- Pass a bootstrap peer via `SAFE_PEERS`. This *has* to be the websocket address,
  e.g. `/ip4/<ip>/tcp/<port>/ws/p2p/<peer ID>`.
    - As well as the other environment variables needed for EVM payments (e.g. `RPC_URL`).
- Optionally specify the specific test, e.g. `-- put` to run `put()` in `wasm.rs` only.

Example:

```sh
SAFE_PEERS=/ip4/<ip>/tcp/<port>/ws/p2p/<peer ID> wasm-pack test --release --firefox autonomi --features=data,files --test wasm -- put
```

#### Test from JS in the browser

`wasm-pack test` does not execute JavaScript, but runs mostly WebAssembly. Again make sure the environment variables are
set and build the JS package:

```sh
wasm-pack build --dev --target=web autonomi --features=vault
```

Then cd into `autonomi/tests-js`, and use `npm` to install and serve the test html file.

```
cd autonomi/tests-js
npm install
npm run serve
```

Then go to `http://127.0.0.1:8080/tests-js` in the browser. Here, enter a `ws` multiaddr of a local node and press '
run'.

#### MetaMask example

There is a MetaMask example for doing a simple put operation.

Build the package with the `external-signer` feature (and again with the env variables) and run a webserver, e.g. with
Python:

```sh
wasm-pack build --dev --target=web autonomi --features=external-signer
python -m http.server --directory=autonomi 8000
```

Then visit `http://127.0.0.1:8000/examples/metamask` in your (modern) browser.

Here, enter a `ws` multiaddr of a local node and press 'run'.

## Faucet (local)

There is no faucet server, but instead you can use the `Deployer wallet private key` printed in the EVM node output to
initialise a wallet from with almost infinite gas and payment tokens. Example:

```rust
let rpc_url = "http://localhost:54370/";
let payment_token_address = "0x5FbDB2315678afecb367f032d93F642f64180aa3";
let data_payments_address = "0x8464135c8F25Da09e49BC8782676a84730C318bC";
let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

let network = Network::Custom(CustomNetwork::new(
rpc_url,
payment_token_address,
data_payments_address,
));

let deployer_wallet = Wallet::new_from_private_key(network, private_key).unwrap();
let receiving_wallet = Wallet::new_with_random_wallet(network);

// Send 10 payment tokens (atto)
let _ = deployer_wallet
.transfer_tokens(receiving_wallet.address(), Amount::from(10))
.await;
```

Alternatively, you can provide the wallet address that should own all the gas and payment tokens to the EVM testnet
startup command using the `--genesis-wallet` flag:

```sh
cargo run --bin evm_testnet -- --genesis-wallet <ETHEREUM_ADDRESS>
```

```shell
*************************
* Ethereum node started *
*************************
RPC URL: http://localhost:60093/
Payment token address: 0x5FbDB2315678afecb367f032d93F642f64180aa3
Chunk payments address: 0x8464135c8F25Da09e49BC8782676a84730C318bC
Deployer wallet private key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
Genesis wallet balance: (tokens: 20000000000000000000000000, gas: 9998998011366954730202)
```

## Python Bindings

The Autonomi client API is also available as a Python package.

### Installation

```bash
pip install autonomi
```

### Basic Usage

```python
from autonomi import Client, Wallet

# Connect to network
client = Client(["/ip4/127.0.0.1/tcp/12000"]) # Change this to the address of a known node

# Create or load a wallet
wallet = Wallet()  # Create new random wallet
# Or load from existing key
wallet = Wallet(secret_key="your-hex-key-here")

# Upload data
data = b"Hello World!"
addr = client.data_put(data, wallet)
print(f"Data stored at: {addr}")

# Retrieve data
retrieved = client.data_get(addr)
```

### Available Classes and Methods

#### Client
- `Client(peers: List[str])` - Connect to the network
- Data Operations:
  - `data_put(data: bytes, wallet: Wallet) -> str` - Store public data
  - `data_get(addr: str) -> bytes` - Retrieve public data
  - `data_cost(data: bytes) -> int` - Get cost to store data
- Private Data:
  - `private_data_put(data: bytes, wallet: Wallet) -> str` - Store private data
  - `private_data_get(access: str) -> bytes` - Retrieve private data
- File Operations:
  - `file_upload(path: str, wallet: Wallet) -> str` - Upload file/directory
  - `file_download(addr: str, path: str)` - Download file
- Register Operations:
  - `register_create(value: bytes, name: str, wallet: Wallet) -> str`
  - `register_get(addr: str) -> List[bytes]`
  - `register_update(addr: str, new_value: bytes, owner: RegisterSecretKey)`
- Vault Operations:
  - `vault_cost(owner: VaultSecretKey) -> int`
  - `get_user_data_from_vault(secret_key: VaultSecretKey) -> UserData`
  - `put_user_data_to_vault(secret_key: VaultSecretKey, wallet: Wallet, user_data: UserData) -> int`

#### Wallet
- `Wallet(secret_key: Optional[str] = None)` - Create new or from existing key
- `to_hex() -> str` - Get hex-encoded secret key
- `address() -> str` - Get wallet address
- `random() -> Wallet` - Create new random wallet
- `from_hex(hex: str) -> Wallet` - Create from hex key
- `network() -> str` - Get network type (mainnet/testnet)

#### Archive and PrivateArchive
- `Archive()` / `PrivateArchive()` - Create new archive
- `add_file(path: str, addr: str, meta: Optional[Metadata])`
- `add_new_file(path: str, addr: str)`
- `files() -> List[Tuple[str, Metadata]]`
- `addresses() -> List[str]` / `access_keys() -> List[str]`
- `rename_file(old_path: str, new_path: str)`

#### UserData
- `UserData()` - Create new user data store
- `register_sk() -> Optional[str]`
- `registers() -> Dict[str, str]`
- `file_archives() -> Dict[str, str]`
- `private_file_archives() -> Dict[str, str]`
- Archive Management:
  - `add_file_archive(archive: str) -> Optional[str]`
  - `add_file_archive_with_name(archive: str, name: str) -> Optional[str]`
  - `add_private_file_archive(archive: str) -> Optional[str]`
  - `add_private_file_archive_with_name(archive: str, name: str) -> Optional[str]`
  - `remove_file_archive(archive: str) -> Optional[str]`
  - `remove_private_file_archive(archive: str) -> Optional[str]`

### Examples

#### Private Data Storage
```python
from autonomi import Client, Wallet

client = Client(["/ip4/127.0.0.1/tcp/12000"])
wallet = Wallet()

# Store private data
secret = b"My secret data"
access_key = client.private_data_put(secret, wallet)
print(f"Access key: {access_key}")

# Retrieve private data
retrieved = client.private_data_get(access_key)
assert retrieved == secret
```

#### Working with Archives
```python
from autonomi import Client, Wallet, Archive, Metadata

client = Client(["/ip4/127.0.0.1/tcp/12000"])
wallet = Wallet()

# Create and populate archive
archive = Archive()
data = b"File content"
addr = client.data_put(data, wallet)
archive.add_file("example.txt", addr, Metadata())

# Store archive
archive_addr = client.archive_put(archive, wallet)

# Retrieve archive
retrieved = client.archive_get(archive_addr)
for path, meta in retrieved.files():
    print(f"File: {path}, uploaded: {meta.uploaded}")
```

#### Vault and User Data
```python
from autonomi import Client, Wallet, VaultSecretKey, UserData

client = Client(["/ip4/127.0.0.1/tcp/12000"])
wallet = Wallet()

# Create vault
vault_key = VaultSecretKey.generate()
cost = client.vault_cost(vault_key)
print(f"Vault creation will cost: {cost}")

# Store user data
user_data = UserData()
user_data.add_file_archive("some_archive_addr", "My Files")
cost = client.put_user_data_to_vault(vault_key, wallet, user_data)

# Retrieve user data
retrieved = client.get_user_data_from_vault(vault_key)
for addr, name in retrieved.file_archives().items():
    print(f"Archive: {name} at {addr}")
```
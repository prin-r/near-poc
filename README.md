# Std-Reference-Basic and Std-Proxy example in Rust

## Setup

Install near-cli

```
nvm use 13
npm install near-cli -g
near --version
```

Build & Deploy `std_ref_basic`

```
cd std_ref_basic
RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
near deploy --wasmFile target/wasm32-unknown-unknown/release/std_reference_basic.wasm --initFunction new --initArgs '{}' --accountId 1.mumu.testnet
```

Build & Deploy `std_proxy`

```
cd std_proxy
RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
near deploy --wasmFile target/wasm32-unknown-unknown/release/std_proxy.wasm --initFunction new --initArgs '{"ref_":"1.mumu.testnet"}' --accountId 2.mumu.testnet
```

## To Test

```
cd std_ref_basic
cargo test -- --nocapture
```

```
cd std_proxy
cargo test -- --nocapture
```

## Interaction

`relay`

```
near call 1.mumu.testnet relay --args '{"symbols": ["BTC", "ETH"], "rates":[777,555], "resolve_times":[11,55], "request_ids":[0,0]}' --accountId mumu.testnet
```

`get_reference_data`

```
near view 1.mumu.testnet get_reference_data --args  '{"base":"BAND","quote":"USD"}' --accountId mumu.testnet
```

`get_reference_data_bulk`

```
near view 1.mumu.testnet get_reference_data_bulk --args  '{"bases":["BTC","ETH","BAND"],"quotes":["USD","USD","USD"]}' --accountId mumu.testnet
```

## Example Js

```
cd example_feeder_js
yarn
node index.js
```

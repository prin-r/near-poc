# Simple Price DB

#### Example deployed contract

- [`simple_price_db.mumu.testnet`](https://explorer.testnet.near.org/accounts/simple_price_db.mumu.testnet)

#### Callable Function

- set_oracle : Set an address of the oracle contract

  ```
  near call simple_price_db.mumu.testnet set_oracle --args '{"new_oracle":"some_oracle.testnet"}' --accountId mumu.testnet --gas 150000000000000
  ```

- set_single : Set price for the given base and quote symbol

  ```
  near call simple_price_db.mumu.testnet set_single --args '{"base":"BTC", "quote":"USD" }' --accountId myaccount.testnet --gas 150000000000000
  ```

- set_multiple : Set price for all given base and quote symbols

  ```
  near call simple_price_db.mumu.testnet set_multiple --args '{"bases":["BTC","ETH"], "quotes":["USD","USD"] }' --accountId mumu.testnet --gas 150000000000000
  ```

#### View Functions

- get_oracle : Get the current oracle address
  
  ```
  near view simple_price_db.mumu.testnet get_oracle --args  '{}'
  ```

- get_price : Get price of a specific base/quote

  ```
  near view simple_price_db.mumu.testnet get_price --args  '{"symbol": "BTC/USD"}'
  ```

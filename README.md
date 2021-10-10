
# Candy Holders

This is far from finished, but can:

- find tokens with a given update authority
- find holders of those tokens

Neither the Rust or Node APIs completely cover everything that needs to be done to accomplish this task & that's why there are to codebases.

Basic usage:

```bash
$ export RPC_HOST="your host here"

$ cd rs

$ cargo run -q -- -r $RPC_HOST mine-tokens-by-update-authority --update-authority EbR4788Gi79GwcT8cANSq4aDHoxD7XrQVGgCfUiML2wX

$ sqlite3 candy-holders.db 'select token_address from tokens' > metabaes-tokens.log

$ gh gist create metabaes-tokens.log

$ cd ../ts

$ sqlite3 ../rs/candy-holders.db "select token_address from tokens;" | npx ts-node src/index.ts --chill 10 --rpc-host $RPC_HOST | tee metabaes-holders.log 

$ gh gist create metabaes-holders.log
```
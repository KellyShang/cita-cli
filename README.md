# cita-cli

A easy-to-use [CITA](https://github.com/cryptape/cita) command line tool.

Just like the relationship between redis_cli and redis.

## Overview

[cita-cli](./cita-cli): a binary project, command line tool.

[cita-tool](./cita-tool): a crate to support cita-cli, of course, can also be used for secondary development, which contains all the methods needed.

## Todo

- ~Send transaction, support sha3hash and blake2b~
- Kill cita process
- Start cita process
- Monitoring status
- Init cita

## Usage

1. clone and build

```bash
$ git clone git@github.com:driftluo/cita-cli.git
$ cd cita-cli
$ cargo build
```

If you want to support both the blake2b and sha3 algorithms, first install the Sodium library, and then

```bash
$ git clone git@github.com:driftluo/cita-cli.git
$ cd cita-cli/cita-cli
$ cargo build --features blake2b_hash
$ cd ..
```

2. use example

If you think that the url specified on the command line is too complex, you can write the env file directly, 
or the corresponding environment variable cli will get it automatically.

- Get chain height
```bash
$ ./target/debug/cita-cli rpc cita_blockNumber --url http://121.196.200.225:1337
{
  "jsonrpc": "2.0",
  "result": "0x1bc7f",
  "id": 1
}
```

- Send transaction
```bash
$ ./target/debug/cita-cli rpc cita_sendTransaction --private-key "352416e1c910e413768c51390dfd791b414212b7b4fe6b1a18f58007fa894214" --code "606060405234156100105760006000fd5b610015565b60e0806100236000396000f30060606040526000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff16806360fe47b114604b5780636d4ce63c14606c576045565b60006000fd5b341560565760006000fd5b606a60048080359060200190919050506093565b005b341560775760006000fd5b607d60a3565b6040518082815260200191505060405180910390f35b8060006000508190909055505b50565b6000600060005054905060b1565b905600a165627a7a72305820942223976c6dd48a3aa1d4749f45ad270915cfacd9c0bf3583c018d4c86f9da20029" --height 111146 --url http://121.196.200.225:1337
{
  "jsonrpc": "2.0",
  "result": {
    "status": "OK",
    "hash": "0x16251c374ee87eae41cbd9203eea481b861738a19c19df9d3c6603b9fbe84478"
  },
  "id": 2
}
```

- Get transaction receipt
```bash
$ ./target/debug/cita-cli rpc eth_getTransactionReceipt --hash "0x16251c374ee87eae41cbd9203eea481b861738a19c19df9d3c6603b9fbe84478" --url http://121.196.200.225:1337
{
  "jsonrpc": "2.0",
  "result": {
    "transactionHash": "0x16251c374ee87eae41cbd9203eea481b861738a19c19df9d3c6603b9fbe84478",
    "logs": [],
    "blockNumber": "0x1b234",
    "transactionIndex": "0x0",
    "cumulativeGasUsed": "0xafc8",
    "gasUsed": "0xafc8",
    "blockHash": "0xca3733ac87fab23dc3c6c9b644631c98a937b369183c44f5743c5179587a3028",
    "root": null,
    "errorMessage": null,
    "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "contractAddress": "0xd9ae0a3b3e856bf5d01061d99721cc4b136d7e26"
  },
  "id": 1
}
```

- Call contract function
```bash
$ ./target/debug/cita-cli rpc cita_sendTransaction --private-key "352416e1c910e413768c51390dfd791b414212b7b4fe6b1a18f58007fa894214" --address "73552bc4e960a1d53013b40074569ea05b950b4d" --code "60fe47b10000000000000000000000000000000000000000000000000000000000000001" --url http://121.196.200.225:1337
{
  "jsonrpc": "2.0",
  "result": {
    "status": "OK",
    "hash": "0x16001b0498c9b80133278a851859b859ada264d2928fd9b2bf0a1ba716079d23"
  },
  "id": 2
}
```

- Get eth-call result
```bash
$ ./target/debug/cita-cli rpc eth_call --url http://121.196.200.225:1337 --to 0xd9ae0a3b3e856bf5d01061d99721cc4b136d7e26 --data 0x6d4ce63c --height latest
{
  "jsonrpc": "2.0",
  "result": "0x0000000000000000000000000000000000000000000000000000000000000001",
  "id": 1
}
```

- Create new key pair
```bash
$ ./target/debug/cita-cli key create
private key: 0x8ee6aa885d9598f9c4e010b659aeecfc3f113beb646166414756568ab656f0f9
pubkey: 0xe407bef7ef0a0e21395c46cc2e1ed324119783d0f4f47b676d95b23991f9065db1aa7a9099e2193160243a02168feb70c62eb8442e45c4b3542a4b3c8c8ac5bd
address: 0xeea5c3cbb32fec85bc9b9bffa65fc027e4b1c6d5
```

- Generate public keys and addresses based on private keys
```bash
./target/debug/cita-cli key from-private-key --private-key 0x993ef0853d7bf1f4c2977457b50ea6b5f8bc2fd829e3ca3e19f6081ddabb07e9
private key: 0x993ef0853d7bf1f4c2977457b50ea6b5f8bc2fd829e3ca3e19f6081ddabb07e9
pubkey: 0xa3cadf91b0ad021eb05eaa1fc2bb66109b3d004808c5cc2a1fb251a881aa12615394bde17dfaea4fb84372344d28a1bd2c4a9b4ab3f5d34ae524e2431ce494b6
address: 0x9dcd6b234e2772c5451fd4ccf7582f4283140697
```

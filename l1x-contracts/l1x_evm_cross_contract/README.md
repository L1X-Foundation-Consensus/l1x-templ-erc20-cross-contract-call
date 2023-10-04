## Compiling the eBPF byte code:

### Step-01: Build the Example Demo Contract `./examples/l1x-evm-cross-contract`

```bash
RUSTFLAGS='-C link-arg=-s' \
  cargo build \
  --release \
  -p l1x-evm-cross-contract \
  --target wasm32-unknown-unknown
```

### Step-02: Generate LLVM IR File from WASM

To generate the LLVM IR file from the WASM binary, use this command:

```bash
cargo run \
  --release \
  -p wasm-llvmir \
  -- \
  ./target/wasm32-unknown-unknown/release/l1x_evm_cross_contract.wasm
```

### Step-03: Build eBPF Object from LLVM IR

To build the eBPF object from LLVM IR, execute the following command:

```bash
./l1x-wasm-llvmir/build_ebpf.sh \
  ./target/wasm32-unknown-unknown/release/l1x_evm_cross_contract.ll
```

**The output contract object file:** `./target/wasm32-unknown-unknown/release/l1x_evm_cross_contract.o`

## Run Demo

**All below steps should be done on Consensus repo:** https://github.com/L1X-Foundation-Consensus/l1x-consensus

### Run a node

1. Run Cassandra
```bash
sudo docker run -e CASSANDRA_USER=cassandra  -e CASSANDRA_PASSWORD=cassandra -p 9042:9042 cassandra:latest
```
2. Run server
```bash
RUST_LOG=info cargo run --bin server -- --dev
```

### Deploy EVM ERC20

1. Go to `cli` directory
2. Setup `cli``
```bash
cargo build
export RUST_LOG=info
export PRIV_KEY=6d657bbe6f7604fb53bc22e0b5285d3e2ad17f64441b2dc19b648933850f9b46
```
3. Deploy ERC20 contract:
```bash
../target/debug/cli --private-key $PRIV_KEY submit-sol --payload-file-path txn-payload/evm_smart_contract_deployment.json
```
Copy the contract instance address from server log messages, `cli` prints incorrect address for EVM contracts. Let's assume ERC20 contract address is `4560a2f822cccf0809e51d58bedc3153596fa3e9`
4. ERC20 token is supplied with maximal amount of tokens. The owner of the tokens is the user who called deployed this ERC20. Search "Transaction nonce" string in server logs, "sender" is the one who deployed the contract. Let's assume the sender's address is `75104938baa47c54a86004ef998cc76c2e616289`

### Deploy L1X Contract

1. Open `txn-payload/smart_contract_deployment.json` and fix the path, Need to use `l1x_evm_cross_contract.o` from the previous steps
2. Deploy
```bash
../target/debug/cli --private-key $PRIV_KEY submit-txn --payload-file-path txn-payload/smart_contract_deployment.json
```
3. Copy the address `cli` is printed and use this address in `txn-payload/smart_contract_init.json` in `hex` field. Let's assume this address is `45ec5100d8199177d3b8634a61b231552d55d8d2`
4. Write initialization arguments. The `smart_contract_init.json` will looks like the following:
```json
{
  "smart_contract_init": [
    { "hex": "45ec5100d8199177d3b8634a61b231552d55d8d2" },
    { "text": "{\"evm_address\":\"4560a2f822cccf0809e51d58bedc3153596fa3e9\"}" }
  ]
}
```
5. Initialize L1X contract
```bash
../target/debug/cli --private-key $PRIV_KEY submit-txn --payload-file-path txn-payload/smart_contract_init.json
```
6. Copy contract instance address `cli` is printed. Let's assume this is `c21ab6058e399622caabd1fd5ba484bd5279c851`

### Run cross-contract call

**1. balance_of**

Check balance of the owner of ERC20 contract (the one who deployed ERC20 in previous steps). Example of `balance_of.json`:
```json
{
  "smart_contract_function_call": {
    "contract_instance_address": { "hex": "c21ab6058e399622caabd1fd5ba484bd5279c851" },
    "function": { "text": "balance_of" },
    "arguments": { "text": "{\"address\":\"75104938baa47c54a86004ef998cc76c2e616289\"}" }
  }
}
```
Run:
```bash
../target/debug/cli --private-key $PRIV_KEY submit-txn --payload-file-path txn-payload/balance_of.json
```
The execution result will be printed to server logs

**2. Fund the contract instance address with ERC20 tokens**

1. Go to https://abi.hashex.org/
2. Fill fields like on the following image but use antoher contract instance address:
![](https://i.imgur.com/mBFkRzn.png)
3. Copy Encoded data content to `arguments` field in `txn-payload/evm_smart_contract_function_call.json`
4. Fix `contract_instance_address` in `txn-payload/evm_smart_contract_function_call.json`. It should ERC20 address. This is `4560a2f822cccf0809e51d58bedc3153596fa3e9` in this readme.
5. Run
```bash
../target/debug/cli --private-key $PRIV_KEY submit-txn --payload-file-path txn-payload/evm_smart_contract_function_call.json
```

**3. transfer**

Transfer tokens from L1X contract instance address to another address. Before calling this function need to fund L1X contract instance.
Example of `transfer.json`:
```json
{
  "smart_contract_function_call": {
    "contract_instance_address": { "hex": "c21ab6058e399622caabd1fd5ba484bd5279c851" },
    "function": { "text": "transfer" },
    "arguments": { "text": "{\"address\":\"45ec5100d8199177d3b8634a61b231552d55d8d2\", \"amount\":\"1\"}" }
  }
}
```
Run:
```bash
../target/debug/cli --private-key $PRIV_KEY submit-txn --payload-file-path txn-payload/transfer.json
```
The execution result will be printed to server logs

**The following ERC20.sol is used in this example:**

```Solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.7.0;
import "@balancer-labs/v2-interfaces/contracts/solidity-utils/openzeppelin/IERC20.sol";

contract MYERC20 is IERC20 {
    string public name;
    string public symbol;
    uint8 public decimals;

    mapping(address => uint256) balances;
    mapping(address => mapping(address => uint256)) allowed;
    uint256 totalSupply_;

    constructor(
        uint256 initialSupply,
        string memory _name,
        string memory _symbol,
        uint8 _decimals
    ) {
        name = _name;
        symbol = _symbol;
        decimals = _decimals;
        totalSupply_ = initialSupply * (10**uint256(decimals));
        balances[msg.sender] = totalSupply_;
        emit Transfer(address(0), msg.sender, totalSupply_);
    }

    function totalSupply() public view override returns (uint256) {
        return totalSupply_;
    }

    function balanceOf(address tokenOwner) public view override returns (uint256) {
        return balances[tokenOwner];
    }

    function transfer(address receiver, uint256 numTokens) public override returns (bool) {
        require(numTokens <= balances[msg.sender]);
        balances[msg.sender] = balances[msg.sender] - numTokens;
        balances[receiver] = balances[receiver] + numTokens;
        emit Transfer(msg.sender, receiver, numTokens);
        return true;
    }

    function approve(address delegate, uint256 numTokens) public override returns (bool) {
        allowed[msg.sender][delegate] = numTokens;
        emit Approval(msg.sender, delegate, numTokens);
        return true;
    }

    function allowance(address owner, address delegate) public view override returns (uint256) {
        return allowed[owner][delegate];
    }

    function transferFrom(
        address owner,
        address buyer,
        uint256 numTokens
    ) public override returns (bool) {
        require(numTokens <= balances[owner]);
        require(numTokens <= allowed[owner][msg.sender]);

        balances[owner] = balances[owner] - numTokens;
        allowed[owner][msg.sender] = allowed[owner][msg.sender] - numTokens;
        balances[buyer] = balances[buyer] + numTokens;
        emit Transfer(owner, buyer, numTokens);
        return true;
    }
}
```

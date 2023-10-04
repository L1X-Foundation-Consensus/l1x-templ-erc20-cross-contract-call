use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use l1x_sdk::call_contract;
use l1x_sdk::contract;
use l1x_sdk::contract_instance_address;
use l1x_sdk::contract_interaction::ContractCall;
use l1x_sdk::types::{Address, U128};
use solabi;

const STORAGE_CONTRACT_KEY: &[u8] = b"STATE";

#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct EvmErc20 {
    evm_contract_address: Address,
}

impl EvmErc20 {
    pub fn new(evm_contract_address: Address) -> Self {
        Self { evm_contract_address }
    }

    pub fn totalSupply(&self) -> solabi::U256 {
        let func: solabi::FunctionEncoder<(), (solabi::U256,)> =
            solabi::FunctionEncoder::new(solabi::selector!("totalSupply()"));

        self.call_evm(&func, &(), true).0
    }

    pub fn balanceOf(&self, tokenOwner: solabi::Address) -> solabi::U256 {
        let func: solabi::FunctionEncoder<(solabi::Address,), (solabi::U256,)> =
            solabi::FunctionEncoder::new(solabi::selector!(
                "balanceOf(address)"
            ));

        self.call_evm(&func, &(tokenOwner,), true).0
    }

    pub fn transfer(
        &self,
        receiver: solabi::Address,
        numTokens: solabi::U256,
    ) -> bool {
        let func: solabi::FunctionEncoder<
            (solabi::Address, solabi::U256),
            (bool,),
        > = solabi::FunctionEncoder::new(solabi::selector!(
            "transfer(address,uint256)"
        ));

        self.call_evm(&func, &(receiver, numTokens), false).0
    }

    pub fn approve(
        &self,
        delegate: solabi::Address,
        numTokens: solabi::U256,
    ) -> bool {
        let func: solabi::FunctionEncoder<
            (solabi::Address, solabi::U256),
            (bool,),
        > = solabi::FunctionEncoder::new(solabi::selector!(
            "approve(address,uint256)"
        ));

        self.call_evm(&func, &(delegate, numTokens), false).0
    }

    pub fn allowance(
        &self,
        owner: solabi::Address,
        delegate: solabi::Address,
    ) -> solabi::U256 {
        let func: solabi::FunctionEncoder<
            (solabi::Address, solabi::Address),
            (solabi::U256,),
        > = solabi::FunctionEncoder::new(solabi::selector!(
            "allowance(address,address)"
        ));

        self.call_evm(&func, &(owner, delegate), true).0
    }

    pub fn transferFrom(
        &self,
        owner: solabi::Address,
        buyer: solabi::Address,
        numTokens: solabi::U256,
    ) -> bool {
        let func: solabi::FunctionEncoder<
            (solabi::Address, solabi::Address, solabi::U256),
            (bool,),
        > = solabi::FunctionEncoder::new(solabi::selector!(
            "transferFrom(address,address,uint256)"
        ));

        self.call_evm(&func, &(owner, buyer, numTokens), true).0
    }

    fn call_evm<P, R>(
        &self,
        func: &solabi::FunctionEncoder<P, R>,
        params: &P,
        read_only: bool,
    ) -> R
    where
        P: solabi::encode::Encode + solabi::decode::Decode,
        R: solabi::encode::Encode + solabi::decode::Decode,
    {
        let args = func.encode_params(params);

        l1x_sdk::msg(&format!("L1XVM: HEX_ARG: {}", hex::encode(&args)));

        let call = ContractCall {
            contract_address: self.evm_contract_address.clone(),
            method_name: "".to_string(), // method_name is not used in case of EVM call
            args,
            read_only,
            fee_limit: 12,
        };

        let ret = call_contract(&call).expect("Function returned nothing");

        func.decode_returns(&ret)
            .unwrap_or_else(|e| panic!("err: {}", e.to_string()))
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Contract {
    evm_erc20: EvmErc20,
}

#[contract]
impl Contract {
    pub fn new(evm_address: Address) {
        let mut contract = Self { evm_erc20: EvmErc20::new(evm_address) };
        Self::save(&mut contract);
    }

    pub fn balance_of(address: Address) -> String {
        l1x_sdk::msg(&format!("L1XVM: balance_of {}", address));

        let contract = Self::load();
        let ret = contract
            .evm_erc20
            .balanceOf(solabi::Address::from_slice(address.as_bytes()))
            .to_string();

        l1x_sdk::msg(&format!(
            "L1XVM: Balance of {} is {} tokens",
            address, ret
        ));
        ret
    }

    pub fn transfer(address: Address, amount: U128) {
        l1x_sdk::msg(&format!(
            "L1XVM: Transfer to: {} amount: {}",
            address, amount.0
        ));

        let contract = Self::load();

        contract.evm_erc20.transfer(
            solabi::Address::from_slice(address.as_bytes()),
            solabi::U256::from(amount.0),
        );
        l1x_sdk::msg(&format!(
            "L1XVM: Transfered from: {} to: {} amount: {}",
            contract_instance_address(),
            address,
            amount.0
        ));
    }

    fn load() -> Self {
        match l1x_sdk::storage_read(STORAGE_CONTRACT_KEY) {
            Some(bytes) => Self::try_from_slice(&bytes).unwrap(),
            None => panic!("The contract isn't initialized"),
        }
    }

    fn save(&mut self) {
        l1x_sdk::storage_write(
            STORAGE_CONTRACT_KEY,
            &self.try_to_vec().unwrap(),
        );
    }
}

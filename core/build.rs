fn main() {
    hub_core_build::run("proto.toml").unwrap();
    evm_contracts_build::run("https://raw.githubusercontent.com/holaplex/hub-evm-contracts/main/abi/contracts/tokens/EditionContract.sol/EditionContract.json").unwrap();
}

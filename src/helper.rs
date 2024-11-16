sol_interface! {
    interface IChronicle  {
        function read() external view;
        function readWithAge() external view;
    }
}


#[public]
impl OracleReader {
    pub fn call_view(&mut self, contract_address: Address) -> Result<Vec<u8>, Vec<u8>> {
        let external_contract = IChronicle::new(contract_address);

        let config = Call::new_in(self);

        let result = external_contract.read(config)?;

        Ok(result)
    }
}
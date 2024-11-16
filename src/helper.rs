sol_interface! {
    interface IChronicle  {
        function read() external view;
        function readWithAge() external view;
    }
}


#[public]
impl OracleReader {
    pub fn call_view(&mut self, contract_address: Address) -> Result<(), Vec<u8>> {
        let external_contract = IChronicle::new(contract_address);
        let config = Call::new_in(self);
        Ok(external_contract.read(config)?)
    }

}
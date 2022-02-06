use workspaces::{Account, Worker, Contract, DevNetwork};

pub mod contract_call_tester {
    use super::*;
    /// Calls a contract method on a contract
    /// 
    /// Receives an expected value
    /// 
    /// Compares returned value against expected and prints outcome
    /// 
    /// # To Do
    /// 
    /// - [ ] add ability to expect an error and parse appropriately
    pub async fn test_contract_call(
        caller: &Account,
        worker: &Worker<impl DevNetwork>,
        contract: &Contract,
        method: &str,
        arguments: serde_json::Value,
        expected_result: &str,
        result_is_serializable: bool
    ) -> anyhow::Result<()> {
        // make contract call
        let result = caller
            .call(
                &worker,
                contract.id().to_owned(),
                method
            )
            .args_json(arguments)?
            .transact()
            .await?;
        
        // for methods which return non-serialized data
        if !result_is_serializable {
            return Ok(());
        }

        // parse result to json
        match result.json::<String>() {
            Ok(result) => {
                if (result == expected_result.to_owned()) == true {
                println!("{} test: passed", method);
                } else {
                    println!("{} test: failed", method);
                    println!("left: {} != right: {}", result, expected_result);
                }
            },
            Err(error) => return Err(error)
        }
        Ok(())
    }
}
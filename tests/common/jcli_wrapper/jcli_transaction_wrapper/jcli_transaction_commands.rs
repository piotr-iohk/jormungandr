#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;

use common::configuration;

#[derive(Debug)]
pub struct TransactionCommands {}

impl TransactionCommands {
    pub fn new() -> TransactionCommands {
        TransactionCommands {}
    }

    pub fn get_new_transaction_command(&self, staging_file: &PathBuf) -> Command {
        let mut command = Command::new(configuration::get_jcli_app().as_os_str());
        command
            .arg("transaction")
            .arg("new")
            .arg("--staging")
            .arg(staging_file.as_os_str());
        command
    }

    pub fn get_add_input_command(
        &self,
        tx_id: &str,
        tx_index: &i32,
        amount: &str,
        staging_file: &PathBuf,
    ) -> Command {
        let mut command = Command::new(configuration::get_jcli_app().as_os_str());
        command
            .arg("transaction")
            .arg("add-input")
            .arg(&tx_id)
            .arg(tx_index.to_string())
            .arg(&amount)
            .arg("--staging")
            .arg(staging_file.as_os_str());
        command
    }

    pub fn get_add_account_command(
        &self,
        account_addr: &str,
        amount: &i32,
        staging_file: &PathBuf,
    ) -> Command {
        let mut command = Command::new(configuration::get_jcli_app().as_os_str());
        command
            .arg("transaction")
            .arg("add-account")
            .arg(account_addr.to_string())
            .arg(amount.to_string())
            .arg("--staging")
            .arg(staging_file.as_os_str());
        command
    }

    pub fn get_add_output_command(
        &self,
        addr: &str,
        amount: &i32,
        staging_file: &PathBuf,
    ) -> Command {
        let mut command = Command::new(configuration::get_jcli_app().as_os_str());
        command
            .arg("transaction")
            .arg("add-output")
            .arg(&addr)
            .arg(amount.to_string())
            .arg("--staging")
            .arg(staging_file.as_os_str());
        command
    }

    pub fn get_finalize_command(&self, staging_file: &PathBuf) -> Command {
        let mut command = Command::new(configuration::get_jcli_app().as_os_str());
        command
            .arg("transaction")
            .arg("finalize")
            .arg("--staging")
            .arg(staging_file.as_os_str());
        command
    }

    pub fn get_make_witness_command(
        &self,
        block0_hash: &str,
        tx_id: &str,
        addr_type: &str,
        spending_account_counter: &i32,
        witness_file: &PathBuf,
        witness_key: &PathBuf,
    ) -> Command {
        let mut command = Command::new(configuration::get_jcli_app().as_os_str());
        command
            .arg("transaction")
            .arg("make-witness")
            .arg("--genesis-block-hash")
            .arg(block0_hash)
            .arg("--type")
            .arg(&addr_type)
            .arg(&tx_id)
            .arg(witness_file.as_os_str())
            .arg("--account-spending-counter")
            .arg(spending_account_counter.to_string())
            .arg(witness_key.as_os_str());
        command
    }

    pub fn get_add_witness_command(
        &self,
        witness_file: &PathBuf,
        staging_file: &PathBuf,
    ) -> Command {
        let mut command = Command::new(configuration::get_jcli_app().as_os_str());
        command
            .arg("transaction")
            .arg("add-witness")
            .arg(witness_file.as_os_str())
            .arg("--staging")
            .arg(staging_file.as_os_str());
        command
    }

    pub fn get_seal_command(&self, staging_file: &PathBuf) -> Command {
        let mut command = Command::new(configuration::get_jcli_app().as_os_str());
        command
            .arg("transaction")
            .arg("seal")
            .arg("--staging")
            .arg(staging_file.as_os_str());
        command
    }

    pub fn get_transaction_message_to_command(&self, staging_file: &PathBuf) -> Command {
        let mut command = Command::new(configuration::get_jcli_app().as_os_str());
        command
            .arg("transaction")
            .arg("to-message")
            .arg("--staging")
            .arg(staging_file.as_os_str());
        command
    }

    pub fn get_transaction_id_command(&self, staging_file: &PathBuf) -> Command {
        let mut command = Command::new(configuration::get_jcli_app().as_os_str());
        command
            .arg("transaction")
            .arg("info")
            .arg("--format")
            .arg("{id}")
            .arg("--staging")
            .arg(staging_file.as_os_str());
        command
    }
}

#![cfg(feature = "integration-test")]

extern crate custom_error;

use self::custom_error::custom_error;
use std::path::PathBuf;

use common::configuration::jormungandr_config::JormungandrConfig;
use common::jcli_wrapper;
use common::jormungandr::{commands, process::JormungandrProcess};

use common::process_assert;
use common::process_utils;
use common::process_utils::{output_extensions::ProcessOutput, ProcessError};
use std::process::{Command, Output};

custom_error! {pub StartupError
    JormungandrNotLaunched{ source: ProcessError } = "could not start jormungandr",
}

fn try_to_start_jormungandr_node(
    rest_address: &str,
    command: &mut Command,
    log_file_path: PathBuf,
) -> Result<JormungandrProcess, StartupError> {
    println!("Starting jormungandr node...");
    let process = command
        .spawn()
        .expect("failed to execute 'start jormungandr node'");

    let guard = JormungandrProcess::new(
        process,
        String::from("Jormungandr node"),
        log_file_path.clone(),
    );

    let proces_start_result = process_utils::run_process_until_response_matches(
        jcli_wrapper::jcli_commands::get_rest_stats_command(&rest_address),
        &is_node_up,
        2,
        5,
        "get stats from jormungandr node",
        "jormungandr node is not up",
    );

    match proces_start_result {
        Ok(_) => return Ok(guard),
        Err(e) => return Err(StartupError::JormungandrNotLaunched { source: e }),
    }
}

fn start_jormungandr_node_sync_with_retry(
    rest_address: &str,
    command: &mut Command,
    config: &mut JormungandrConfig,
) -> JormungandrProcess {
    let first_attempt =
        try_to_start_jormungandr_node(rest_address, command, config.log_file_path.clone());
    match first_attempt {
        Ok(guard) => return guard,
        _ => println!("failed to start jormungandr node. retrying.."),
    };
    config.node_config.regenerate_ports();
    let second_attempt =
        try_to_start_jormungandr_node(rest_address, command, config.log_file_path.clone());
    match second_attempt {
        Ok(guard) => return guard,
        Err(e) => panic!(e.to_string()),
    };
}

fn is_node_up(output: Output) -> bool {
    match output.as_single_node_yaml().get("uptime") {
        Some(uptime) => {
            return uptime
                .parse::<i32>()
                .expect(&format!("Cannot parse uptime {}", uptime.to_string()))
                > 2
        }
        None => return false,
    }
}

pub fn start_jormungandr_node(config: &mut JormungandrConfig) -> JormungandrProcess {
    let rest_address = &config.node_config.get_node_address();

    let mut command = commands::get_start_jormungandr_node_command(
        &config.node_config_path,
        &config.genesis_block_path,
        &config.log_file_path,
    );

    println!("Starting node with configuration : {:?}", &config);
    let process = start_jormungandr_node_sync_with_retry(&rest_address, &mut command, config);
    process
}

pub fn start_jormungandr_node_as_leader(config: &mut JormungandrConfig) -> JormungandrProcess {
    let rest_address = &config.node_config.get_node_address();

    let mut command = commands::get_start_jormungandr_as_leader_node_command(
        &config.node_config_path,
        &config.genesis_block_path,
        &config.secret_model_path,
        &config.log_file_path,
    );

    println!("Starting node with configuration : {:?}", &config);
    let process = start_jormungandr_node_sync_with_retry(&rest_address, &mut command, config);
    process
}

pub fn start_jormungandr_node_as_slave(config: &mut JormungandrConfig) -> JormungandrProcess {
    let rest_address = &config.node_config.get_node_address();

    let mut command = commands::get_start_jormungandr_as_slave_node_command(
        &config.node_config_path,
        &config.genesis_block_hash,
        &config.log_file_path,
    );

    println!("Starting node with configuration : {:?}", &config);
    let process = start_jormungandr_node_sync_with_retry(&rest_address, &mut command, config);
    process
}

pub fn start_jormungandr_node_as_passive(config: &mut JormungandrConfig) -> JormungandrProcess {
    let rest_address = &config.node_config.get_node_address();

    let mut command = commands::get_start_jormungandr_as_passive_node_command(
        &config.node_config_path,
        &config.genesis_block_hash,
        &config.secret_model_path,
        &config.log_file_path,
    );

    println!("Starting node with configuration : {:?}", &config);
    let process = start_jormungandr_node_sync_with_retry(&rest_address, &mut command, config);
    process
}

pub fn assert_start_jormungandr_node_as_passive_fail(
    config: &mut JormungandrConfig,
    expected_msg: &str,
) {
    let command = commands::get_start_jormungandr_as_passive_node_command(
        &config.node_config_path,
        &config.genesis_block_hash,
        &config.secret_model_path,
        &config.log_file_path,
    );

    process_assert::assert_process_failed_and_matches_message(command, &expected_msg);
}

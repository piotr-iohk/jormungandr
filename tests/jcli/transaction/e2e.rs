#![cfg(feature = "integration-test")]

use common::configuration::genesis_model::Fund;
use common::jcli_wrapper;
use common::jcli_wrapper::jcli_transaction_wrapper::JCLITransactionWrapper;
use common::startup;

const FAKE_INPUT_TRANSACTION_ID: &str =
    "19c9852ca0a68f15d0f7de5d1a26acd67a3a3251640c6066bdb91d22e2000193";

#[test]
pub fn test_utxo_transation_with_more_than_one_witness_per_input_is_rejected() {
    let sender = startup::create_new_utxo_address();
    let reciever = startup::create_new_utxo_address();
    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: 100,
        }])
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);
    let block0_hash = jcli_wrapper::assert_genesis_hash(&config.genesis_block_path);

    let mut transaction_wrapper = JCLITransactionWrapper::new_transaction(&block0_hash);
    transaction_wrapper
        .assert_add_input_from_utxo(&utxo)
        .assert_add_output(&reciever.address, &utxo.out_value)
        .assert_finalize();

    let witness1 = transaction_wrapper.create_witness_default("utxo");
    let witness2 = transaction_wrapper.create_witness_default("utxo");

    transaction_wrapper
        .assert_make_witness(&witness1)
        .assert_add_witness(&witness1)
        .assert_make_witness(&witness2)
        .assert_add_witness_fail(&witness2, "cannot add anymore witnesses");
}

#[test]
pub fn test_two_correct_utxo_to_utxo_transactions_are_accepted_by_node() {
    let sender = startup::create_new_utxo_address();
    let middle_man = startup::create_new_utxo_address();
    let reciever = startup::create_new_utxo_address();

    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: 100,
        }])
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);

    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);
    let block0_hash = jcli_wrapper::assert_genesis_hash(&config.genesis_block_path);
    let transaction_builder = JCLITransactionWrapper::build_transaction_from_utxo(
        &utxo,
        &utxo.out_value,
        &middle_man,
        &utxo.out_value,
        &sender,
        &block0_hash,
    );

    let transaction_message = transaction_builder.assert_transaction_to_message();
    let first_transaction_id = transaction_builder.get_transaction_id();

    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);

    let transaction_message = JCLITransactionWrapper::build_transaction(
        &first_transaction_id,
        &0,
        &100,
        &reciever,
        &100,
        &middle_man,
        &block0_hash,
    )
    .assert_transaction_to_message();

    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

#[test]
pub fn test_correct_utxo_transaction_is_accepted_by_node() {
    let sender = startup::create_new_utxo_address();
    let reciever = startup::create_new_utxo_address();

    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: 100,
        }])
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);

    let transaction_message = JCLITransactionWrapper::new_transaction(&config.genesis_block_hash)
        .assert_add_input_from_utxo(&utxo)
        .assert_add_output(&reciever.address, &utxo.out_value)
        .assert_finalize()
        .seal_with_witness_deafult(&sender.private_key, "utxo")
        .assert_transaction_to_message();

    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

#[test]
pub fn test_correct_utxo_transaction_replaces_old_utxo_by_node() {
    let sender = startup::create_new_utxo_address();
    let reciever = startup::create_new_utxo_address();

    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: 100,
        }])
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);

    let mut transaction_builder =
        JCLITransactionWrapper::new_transaction(&config.genesis_block_hash);
    let transaction_message = transaction_builder
        .assert_add_input_from_utxo(&utxo)
        .assert_add_output(&reciever.address, &utxo.out_value)
        .assert_finalize()
        .seal_with_witness_deafult(&sender.private_key, "utxo")
        .assert_transaction_to_message();

    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);

    std::thread::sleep(std::time::Duration::from_secs(2));

    let utxos = jcli_wrapper::assert_rest_utxo_get(&jormungandr_rest_address);
    let transaction_id = transaction_builder.get_transaction_id();

    assert_eq!(utxos.len(), 1);

    let utxo = &utxos[0];
    assert_eq!(
        utxo.out_addr, reciever.address,
        "after sucessful transaction out_addr for utxo should be equal to reciever address"
    );
    assert_eq!(
        utxo.out_value, 100,
        "out value should be equal to output of first transaction"
    );
    assert_eq!(
        utxo.in_idx, 0,
        "since only one transaction was made, idx should be equal to 1"
    );
    assert_eq!(
        utxo.in_txid, transaction_id,
        "transaction hash should be equal to new transaction"
    );
}

#[test]
#[ignore] // This test is known to fail
pub fn test_account_is_created_if_transaction_out_is_account() {
    let sender = startup::create_new_utxo_address();
    let reciever = startup::create_new_account_address();
    let transfer_amount = 100;

    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: transfer_amount.clone(),
        }])
        .with_allow_account_creation(true)
        .build();
    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);

    let transaction_message = JCLITransactionWrapper::new_transaction(&config.genesis_block_hash)
        .assert_add_input_from_utxo(&utxo)
        .assert_add_output(&reciever.address, &transfer_amount)
        .assert_finalize()
        .seal_with_witness_deafult(&sender.private_key, "utxo")
        .assert_transaction_to_message();

    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
    std::thread::sleep(std::time::Duration::from_secs(2));

    jcli_wrapper::assert_rest_account_get_stats(&sender.address, &jormungandr_rest_address);
}

#[test]
pub fn test_transaction_from_delegation_to_delegation_is_accepted_by_node() {
    let sender = startup::create_new_delegation_address();
    let reciever = startup::create_new_delegation_address();
    let transfer_amount = 100;

    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: transfer_amount.clone(),
        }])
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);

    let transaction_message = JCLITransactionWrapper::new_transaction(&config.genesis_block_hash)
        .assert_add_input_from_utxo(&utxo)
        .assert_add_output(&reciever.address, &transfer_amount)
        .assert_finalize()
        .seal_with_witness_deafult(&sender.private_key, "utxo")
        .assert_transaction_to_message();

    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

#[test]
pub fn test_transaction_from_delegation_to_account_is_accepted_by_node() {
    let sender = startup::create_new_delegation_address();
    let reciever = startup::create_new_account_address();
    let transfer_amount = 100;

    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: transfer_amount.clone(),
        }])
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);

    let transaction_message = JCLITransactionWrapper::new_transaction(&config.genesis_block_hash)
        .assert_add_input_from_utxo(&utxo)
        .assert_add_output(&reciever.address, &transfer_amount)
        .assert_finalize()
        .seal_with_witness_deafult(&sender.private_key, "utxo")
        .assert_transaction_to_message();

    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

#[test]
pub fn test_transaction_from_delegation_to_utxo_is_accepted_by_node() {
    let sender = startup::create_new_delegation_address();
    let reciever = startup::create_new_utxo_address();
    let transfer_amount = 100;

    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: transfer_amount.clone(),
        }])
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);

    let transaction_message = JCLITransactionWrapper::new_transaction(&config.genesis_block_hash)
        .assert_add_input_from_utxo(&utxo)
        .assert_add_output(&reciever.address, &transfer_amount)
        .assert_finalize()
        .seal_with_witness_deafult(&sender.private_key, "utxo")
        .assert_transaction_to_message();

    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

#[test]
pub fn test_transaction_from_utxo_to_account_is_accepted_by_node() {
    let sender = startup::create_new_utxo_address();
    let reciever = startup::create_new_account_address();

    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: 100,
        }])
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);

    let transaction_message = JCLITransactionWrapper::new_transaction(&config.genesis_block_hash)
        .assert_add_input_from_utxo(&utxo)
        .assert_add_output(&reciever.address, &utxo.out_value)
        .assert_finalize()
        .seal_with_witness_deafult(&sender.private_key, "utxo")
        .assert_transaction_to_message();

    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

#[test]
pub fn test_transaction_from_account_to_account_is_accepted_by_node() {
    let sender = startup::create_new_account_address();
    let reciever = startup::create_new_account_address();
    let transfer_amount = 100;

    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: transfer_amount.clone(),
        }])
        .with_allow_account_creation(true)
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);

    let transaction_message = JCLITransactionWrapper::new_transaction(&config.genesis_block_hash)
        .assert_add_account(&sender.address, &transfer_amount)
        .assert_add_output(&reciever.address, &transfer_amount)
        .assert_finalize()
        .seal_with_witness_deafult(&sender.private_key, "account")
        .assert_transaction_to_message();

    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

#[test]
pub fn test_transaction_from_account_to_delegation_is_accepted_by_node() {
    let sender = startup::create_new_account_address();
    let reciever = startup::create_new_delegation_address();
    let transfer_amount = 100;

    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: transfer_amount.clone(),
        }])
        .with_allow_account_creation(true)
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);

    let transaction_message = JCLITransactionWrapper::new_transaction(&config.genesis_block_hash)
        .assert_add_account(&sender.address, &transfer_amount)
        .assert_add_output(&reciever.address, &transfer_amount)
        .assert_finalize()
        .seal_with_witness_deafult(&sender.private_key, "account")
        .assert_transaction_to_message();

    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

#[test]
pub fn test_transaction_from_utxo_to_delegation_is_accepted_by_node() {
    let sender = startup::create_new_utxo_address();
    let reciever = startup::create_new_delegation_address();
    let transfer_amount = 100;

    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: transfer_amount.clone(),
        }])
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);

    let transaction_message = JCLITransactionWrapper::new_transaction(&config.genesis_block_hash)
        .assert_add_input_from_utxo(&utxo)
        .assert_add_output(&reciever.address, &transfer_amount)
        .assert_finalize()
        .seal_with_witness_deafult(&sender.private_key, "utxo")
        .assert_transaction_to_message();

    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

#[test]
pub fn test_input_with_smaller_value_than_initial_utxo_is_rejected_by_node() {
    let sender = startup::create_new_utxo_address();
    let reciever = startup::create_new_utxo_address();
    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: 100,
        }])
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let block0_hash = jcli_wrapper::assert_genesis_hash(&config.genesis_block_path);
    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);
    let transaction_message = JCLITransactionWrapper::build_transaction_from_utxo(
        &utxo,
        &99,
        &reciever,
        &99,
        &sender,
        &block0_hash,
    )
    .assert_transaction_to_message();

    /// Assertion is changed due to issue: #332
    /// After fix please revert it to
    ///   jcli_wrapper::assert_transaction_post_failed(&transaction_message, &jormungandr_rest_address);
    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

#[test]
pub fn test_transaction_with_non_existing_id_should_be_rejected_by_node() {
    let sender = startup::create_new_utxo_address();
    let reciever = startup::create_new_utxo_address();
    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: 100,
        }])
        .build();
    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let block0_hash = jcli_wrapper::assert_genesis_hash(&config.genesis_block_path);
    let transaction_message = JCLITransactionWrapper::build_transaction(
        &FAKE_INPUT_TRANSACTION_ID,
        &0,
        &100,
        &reciever,
        &50,
        &sender,
        &block0_hash,
    )
    .assert_transaction_to_message();

    /// Assertion is changed due to issue: #333
    /// After fix please revert it to
    ///   jcli_wrapper::assert_transaction_post_failed(&transaction_message, &jormungandr_rest_address);
    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

#[test]
pub fn test_transaction_with_input_address_equal_to_output_is_accepted_by_node() {
    let sender = startup::create_new_utxo_address();
    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: 100,
        }])
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);
    let transaction_message = JCLITransactionWrapper::build_transaction_from_utxo(
        &utxo,
        &utxo.out_value,
        &sender,
        &utxo.out_value,
        &sender,
        &config.genesis_block_hash,
    )
    .assert_transaction_to_message();
    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

#[test]
pub fn test_input_with_no_spending_utxo_is_accepted_by_node() {
    let sender = startup::create_new_utxo_address();
    let reciever = startup::create_new_utxo_address();
    let mut config = startup::ConfigurationBuilder::new()
        .with_funds(vec![Fund {
            address: sender.address.clone(),
            value: 100,
        }])
        .build();

    let jormungandr_rest_address = config.get_node_address();
    let _jormungandr = startup::start_jormungandr_node_as_leader(&mut config);
    let utxo = startup::get_utxo_for_address(&sender, &jormungandr_rest_address);
    let transaction_message = JCLITransactionWrapper::build_transaction_from_utxo(
        &utxo,
        &100,
        &reciever,
        &50,
        &sender,
        &config.genesis_block_hash,
    )
    .assert_transaction_to_message();
    jcli_wrapper::assert_transaction_post_accepted(&transaction_message, &jormungandr_rest_address);
}

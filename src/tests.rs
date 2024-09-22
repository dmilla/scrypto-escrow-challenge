use scrypto::prelude::*;
use scrypto_test::prelude::*;

use crate::{EscrowBadge, EscrowResourceSpecifier, DEFAULT_ESCROW_ID};

struct TestEnvironment {
    ledger: LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    account1: ComponentAddress,
    account2: ComponentAddress,
    public_key1: Secp256k1PublicKey,
    public_key2: Secp256k1PublicKey,
    offered_resource: ResourceAddress,
    requested_resource: ResourceAddress,
}

fn setup() -> TestEnvironment {
    // Setup the environment
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Create accounts
    let (public_key1, _private_key1, account1) = ledger.new_allocated_account();
    let (public_key2, _private_key2, account2) = ledger.new_allocated_account();

    // Publish package
    let package_address = ledger.compile_and_publish(this_package!());

    // Create test resources
    let offered_resource = ledger.create_fungible_resource(dec!("100"), 0, account1);
    let requested_resource = ledger.create_fungible_resource(dec!("100"), 0, account2);

    TestEnvironment {
        ledger,
        package_address,
        account1,
        account2,
        public_key1,
        public_key2,
        offered_resource,
        requested_resource,
    }
}

fn instantiate_escrow(env: &mut TestEnvironment) -> (ComponentAddress, ResourceAddress) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(env.account1, env.offered_resource, dec!(10))
        .take_all_from_worktop(env.offered_resource, "offered_bucket")
        .call_function_with_name_lookup(
            env.package_address,
            "Escrow",
            "instantiate_escrow",
            |lookup| (
                EscrowResourceSpecifier::Fungible {
                    resource_address: env.requested_resource,
                    amount: dec!("10")
                },
                lookup.bucket("offered_bucket")
            )
        )
        .deposit_batch(env.account1)
        .build();
    let receipt = env.ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&env.public_key1)],
    );
    let result = receipt.expect_commit(true);
    (result.new_component_addresses()[0], result.new_resource_addresses()[0])
}

#[test]
fn test_instantiate_escrow() {
    let mut env = setup();
    let (component, escrow_nft) = instantiate_escrow(&mut env);

    // Assert that the component and escrow_nft were created
    assert!(Some(component) != None);
    assert!(Some(escrow_nft) != None);
    // Assert that the escrow_nft has the correct ID
    let escrow_badge: EscrowBadge = env.ledger.get_non_fungible_data(escrow_nft, NonFungibleLocalId::integer(DEFAULT_ESCROW_ID));
    assert_eq!(escrow_badge.offered_resource, env.offered_resource);
}

#[test]
fn test_successful_exchange() {
    let mut env = setup();
    let (component, _) = instantiate_escrow(&mut env);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(env.account2, env.requested_resource, dec!(10))
        .take_all_from_worktop(env.requested_resource, "requested_bucket")
        .call_method_with_name_lookup(
            component,
            "exchange",
            |lookup| (
                lookup.bucket("requested_bucket"),
            )
        )
        .deposit_batch(env.account2)
        .build();
    let receipt = env.ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&env.public_key2)],
    );
    receipt.expect_commit_success();
}

#[test]
fn test_withdraw_resource() {
    let mut env = setup();
    let (component, escrow_nft) = instantiate_escrow(&mut env);

    // First, perform the exchange
    let exchange_manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(env.account2, env.requested_resource, dec!(10))
        .take_all_from_worktop(env.requested_resource, "requested_bucket")
        .call_method_with_name_lookup(
            component,
            "exchange",
            |lookup| (
                lookup.bucket("requested_bucket"),
            )
        )
        .deposit_batch(env.account2)
        .build();
    env.ledger.execute_manifest(
        exchange_manifest,
        vec![NonFungibleGlobalId::from_public_key(&env.public_key2)],
    ).expect_commit_success();

    // Then, withdraw the resource
    let withdraw_manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_non_fungibles_from_account(env.account1, escrow_nft, [NonFungibleLocalId::integer(DEFAULT_ESCROW_ID)])
        .take_all_from_worktop(escrow_nft, "escrow_nft")
        .call_method_with_name_lookup(
            component,
            "withdraw_resource",
            |lookup| (
                lookup.bucket("escrow_nft"),
            )
        )
        .deposit_batch(env.account1)
        .build();
    let receipt = env.ledger.execute_manifest(
        withdraw_manifest,
        vec![NonFungibleGlobalId::from_public_key(&env.public_key1)],
    );
    receipt.expect_commit_success();
}

#[test]
fn test_cancel_escrow() {
    let mut env = setup();
    let (component, escrow_nft) = instantiate_escrow(&mut env);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_non_fungibles_from_account(env.account1, escrow_nft, [NonFungibleLocalId::integer(DEFAULT_ESCROW_ID)])
        .take_all_from_worktop(escrow_nft, "escrow_nft")
        .call_method_with_name_lookup(
            component,
            "cancel_escrow",
            |lookup| (
                lookup.bucket("escrow_nft"),
            )
        )
        .deposit_batch(env.account1)
        .build();
    let receipt = env.ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&env.public_key1)],
    );
    receipt.expect_commit_success();
}

#[test]
fn test_exchange_with_incorrect_amount() {
    let mut env = setup();
    let (component, _) = instantiate_escrow(&mut env);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(env.account2, env.requested_resource, dec!(5))
        .take_all_from_worktop(env.requested_resource, "requested_bucket")
        .call_method_with_name_lookup(
            component,
            "exchange",
            |lookup| (
                lookup.bucket("requested_bucket"),
            )
        )
        .deposit_batch(env.account2)
        .build();
    let receipt = env.ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&env.public_key2)],
    );
    receipt.expect_commit_failure();
}

#[test]
fn test_withdraw_without_exchange() {
    let mut env = setup();
    let (component, escrow_nft) = instantiate_escrow(&mut env);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_non_fungibles_from_account(env.account1, escrow_nft, [NonFungibleLocalId::integer(DEFAULT_ESCROW_ID)])
        .take_all_from_worktop(escrow_nft, "escrow_nft")
        .call_method_with_name_lookup(
            component,
            "withdraw_resource",
            |lookup| (
                lookup.bucket("escrow_nft"),
            )
        )
        .deposit_batch(env.account1)
        .build();
    let receipt = env.ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&env.public_key1)],
    );
    println!("Withdraw Without Exchange Receipt: {:?}\n", receipt);
    receipt.expect_commit_failure();

}
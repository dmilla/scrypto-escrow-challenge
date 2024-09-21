use scrypto::prelude::*;
use scrypto_test::prelude::*;

use crate::{escrow::Escrow, EscrowResourceSpecifier};

#[test]
fn escrow_test() -> Result<(), RuntimeError> {
    // Setup the environment
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Create an account
    let (public_key, _private_key, account) = ledger.new_allocated_account();

    // Publish package
    let package_address = ledger.compile_and_publish(this_package!());

    // Arrange
    let mut env = TestEnvironment::new();

    // Act
    let offered_resource = ledger.create_fungible_resource(dec!("10"), 0, account);

    let requested_resource: EscrowResourceSpecifier = EscrowResourceSpecifier::Fungible {
        resource_address: XRD,
        amount: dec!("10")
    };
    
    // // Assert
    // let amount = offered_bucket.amount(&mut env).unwrap();
    // assert_eq!(amount, dec!("10"));

    // Below approach seems not to work because of not implemented for non-wasm targets,
    //and wasm32-unknown-unknown target was breaking build
    // ----
    // Test the `instantiate_escrow` function.
    // let instanciate_result = Escrow::instantiate_escrow(
    //     requested_resource,
    //     offered_bucket
    // );

    // // Act
    // // let (pool_units, _change) = radiswap.add_liquidity(bucket1, bucket2, &mut env)?;
    // // Assert
    // let (escrow_component, nft_bucket) = instanciate_result;
    // assert_eq!(nft_bucket.non_fungible_local_id(), IntegerNonFungibleLocalId::new(1).value().into());

    // Test the `instantiate_hello` function.
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, offered_resource, dec!(10))
        .take_all_from_worktop(offered_resource, "offered_bucket")
        .call_function_with_name_lookup(
            package_address,
            "Escrow",
            "instantiate_escrow",
            |lookup| (  // #2
                requested_resource,
                lookup.bucket("offered_bucket")// #3
            )
        )
        .deposit_batch(account)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!("{:?}\n", receipt);
    let component = receipt.expect_commit(true).new_component_addresses()[0];
    println!("Component: {:?}", component);
    Ok(())
}
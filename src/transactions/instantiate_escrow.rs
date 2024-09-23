use escrow_exercise_boilerplate::EscrowResourceSpecifier;
use radix_transactions::{manifest::dumper::dump_manifest_to_file_system, prelude::ManifestBuilder};
use scrypto::prelude::*;

/// An example manifest that creates a new non-fungible resource.
/// 
/// /// A manifest builder for use in tests.
///
/// Note - if you break invariants of the manifest builder (eg resolve a bucket
/// before it's been created, or pass an invalid parameter), this builder will panic.
/// As such, it's only designed for use in test code, or where the inputs are trusted.
///
/// Simple use case:
/// ```
/// # use radix_transactions::prelude::*;
/// # let from_account_address = ComponentAddress::virtual_account_from_public_key(
/// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
/// # );
/// # let to_account_address = ComponentAddress::virtual_account_from_public_key(
/// #   &Ed25519PublicKey([1; Ed25519PublicKey::LENGTH])
/// # );
/// let manifest = ManifestBuilder::new()
///     .lock_fee_from_faucet()
///     .withdraw_from_account(from_account_address, XRD, dec!(1))
///     .take_from_worktop(XRD, dec!(1), "xrd")
///     .try_deposit_or_abort(to_account_address, None, "xrd")
///     .build();
/// ```
///
/// Intermediate use case, where we need to pass a bucket into a component:
/// ```
/// # use radix_transactions::prelude::*;
/// # let package_address = RESOURCE_PACKAGE; // Just some address to get it to compile
/// # let from_account_address = ComponentAddress::virtual_account_from_public_key(
/// #   &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])
/// # );
/// # let to_account_address = ComponentAddress::virtual_account_from_public_key(
/// #   &Ed25519PublicKey([1; Ed25519PublicKey::LENGTH])
/// # );
/// let manifest = ManifestBuilder::new()
///     .lock_fee_from_faucet()
///     .withdraw_from_account(from_account_address, XRD, dec!(1))
///     .take_from_worktop(XRD, dec!(1), "xrd")
///     .call_function_with_name_lookup(
///         package_address,
///         "SomeBlueprint",
///         "some_function",
///         |lookup| (
///             lookup.bucket("xrd"),
///         ),
///     )
///     .build();
/// ```
///
/// Advanced use case, where we need to generate a collision-free bucket name:
/// ```
/// # use radix_transactions::prelude::*;
/// # let to_account_address = ComponentAddress::virtual_account_from_public_key(
/// #   &Ed25519PublicKey([1; Ed25519PublicKey::LENGTH])
/// # );
/// let mut builder = ManifestBuilder::new()
///     .lock_fee_from_faucet()
///     .get_free_xrd_from_faucet();
/// for _ in 0..32 {
///     // The generate_bucket_name method generates a new bucket name starting with
///     // "transfer" that doesn't collide with any previously used bucket names
///     let bucket_name = builder.generate_bucket_name("transfer");
///     builder = builder
///         .take_from_worktop(XRD, "0.001", &bucket_name)
///         .try_deposit_or_abort(to_account_address, None, bucket_name);
/// }
/// let manifest = builder.build();
/// ```
/// 
/// 
fn main() {
    let network = NetworkDefinition::stokenet();

    let decoder =AddressBech32Decoder::new(&network);

    let from_account_address = ComponentAddress::try_from_bech32(&decoder, "account_tdx_2_12xh47xjynaa57nf4wp9xkvcxaasdle0d9w4gglxuce789dz3tffkzx")
            .expect("Invalid from account address");
           
    let package_address = PackageAddress::try_from_bech32(&decoder, "package_tdx_2_1p5emmw82zqhx0ufhplc08u4me8g32llvxxerp9jk2fnmdl2e6ma32y")
           .expect("Invalid package address");

    let manifest = ManifestBuilder::new()
        // Locking fees from the fee payer's account.
        .lock_fee_from_faucet()
        // Creating the resource.
        .withdraw_from_account(from_account_address, XRD, dec!(1))
        .take_from_worktop(XRD, dec!(1), "xrd")
        .call_function_with_name_lookup(
            package_address,
            "Escrow",
            "instantiate_escrow",
            |lookup| (
                EscrowResourceSpecifier::Fungible { resource_address: XRD, amount: dec!("2") },
                lookup.bucket("xrd"),
            ),
        )
        .deposit_batch(from_account_address);

    dump_manifest_to_file_system(
        manifest.object_names(),
        &manifest.build(),
        "./transaction_manifest",
        Some("instantiate_escrow"),
        &network
    ).err();
}
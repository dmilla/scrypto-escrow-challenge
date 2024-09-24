use escrow_exercise_boilerplate::EscrowResourceSpecifier;
use radix_transactions::{manifest::dumper::dump_manifest_to_file_system, prelude::ManifestBuilder};
use scrypto::prelude::*;

fn main() {
    let network = NetworkDefinition::stokenet();

    let decoder =AddressBech32Decoder::new(&network);

    let from_account_address = ComponentAddress::try_from_bech32(&decoder, "account_tdx_2_12xh47xjynaa57nf4wp9xkvcxaasdle0d9w4gglxuce789dz3tffkzx")
            .expect("Invalid from account address");

    let requested_nft_address  = ResourceAddress::try_from_bech32(&decoder, "resource_tdx_2_1ntxn2zuu59fhetlg6xcvm0zpe3naa9pcwt7mpwc6hhkm9qq9myddrs")
    .expect("Invalid badge address");
           
    let package_address = PackageAddress::try_from_bech32(&decoder, "package_tdx_2_1p5emmw82zqhx0ufhplc08u4me8g32llvxxerp9jk2fnmdl2e6ma32y")
           .expect("Invalid package address");

    let manifest = ManifestBuilder::new()
        // Creating the resource.
        .withdraw_from_account(from_account_address, XRD, dec!(5))
        .take_from_worktop(XRD, dec!(5), "xrd")
        .call_function_with_name_lookup(
            package_address,
            "Escrow",
            "instantiate_escrow",
            |lookup| (
                EscrowResourceSpecifier::NonFungible { resource_address: requested_nft_address, non_fungible_local_id: NonFungibleLocalId::integer(0) },
                lookup.bucket("xrd"),
            ),
        )
        .deposit_batch(from_account_address);

    dump_manifest_to_file_system(
        manifest.object_names(),
        &manifest.build(),
        "./transaction_manifest",
        Some("instantiate_escrow_request_nft"),
        &network
    ).err();
}
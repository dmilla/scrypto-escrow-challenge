use radix_transactions::{manifest::dumper::dump_manifest_to_file_system, prelude::ManifestBuilder};
use scrypto::prelude::*;
fn main() {
    let network = NetworkDefinition::stokenet();

    let decoder =AddressBech32Decoder::new(&network);

    let from_account_address = ComponentAddress::try_from_bech32(&decoder, "account_tdx_2_12xh47xjynaa57nf4wp9xkvcxaasdle0d9w4gglxuce789dz3tffkzx")
            .expect("Invalid from account address");
           
    let component_address = ComponentAddress::try_from_bech32(&decoder, "component_tdx_2_1cpeer9jteykrff5hng6uwv4zruud9s4f329k3uv3y5lyjds2wjkxt9")
           .expect("Invalid component address");

    let requested_nft_address  = ResourceAddress::try_from_bech32(&decoder, "resource_tdx_2_1ntxn2zuu59fhetlg6xcvm0zpe3naa9pcwt7mpwc6hhkm9qq9myddrs")
           .expect("Invalid badge address");

    let manifest = ManifestBuilder::new()
        // Locking fees from the fee payer's account.
        .withdraw_non_fungibles_from_account(from_account_address, requested_nft_address, [NonFungibleLocalId::integer(0)])
        .take_all_from_worktop(requested_nft_address, "requested_nft")
        .call_method_with_name_lookup(
            component_address,
            "exchange",
            |lookup| (
                lookup.bucket("requested_nft"),
            )
        )
        .deposit_batch(from_account_address);

    dump_manifest_to_file_system(
        manifest.object_names(),
        &manifest.build(),
        "./transaction_manifest",
        Some("exchange_nft"),
        &network
    ).err();
}
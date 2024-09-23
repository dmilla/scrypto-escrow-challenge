use radix_transactions::{manifest::dumper::dump_manifest_to_file_system, prelude::ManifestBuilder};
use scrypto::prelude::*;
fn main() {
    let network = NetworkDefinition::stokenet();

    let decoder =AddressBech32Decoder::new(&network);

    let from_account_address = ComponentAddress::try_from_bech32(&decoder, "account_tdx_2_12xh47xjynaa57nf4wp9xkvcxaasdle0d9w4gglxuce789dz3tffkzx")
            .expect("Invalid from account address");
           
    let component_address = ComponentAddress::try_from_bech32(&decoder, "component_tdx_2_1cq95llglldhwa8gtmq7kdzqr6mvn02y42qj8p7qptnl2gvfrwswvww")
           .expect("Invalid component address");

    let manifest = ManifestBuilder::new()
        // Locking fees from the fee payer's account.
        .withdraw_from_account(from_account_address, XRD, dec!(5))
        .take_from_worktop(XRD, dec!(5), "xrd")
        .call_method_with_name_lookup(
            component_address,
            "exchange",
            |lookup| (
                lookup.bucket("xrd"),
            )
        )
        .deposit_batch(from_account_address);

    dump_manifest_to_file_system(
        manifest.object_names(),
        &manifest.build(),
        "./transaction_manifest",
        Some("exchange"),
        &network
    ).err();
}
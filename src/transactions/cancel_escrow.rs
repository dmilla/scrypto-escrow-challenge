use radix_transactions::{manifest::dumper::dump_manifest_to_file_system, prelude::ManifestBuilder};
use scrypto::prelude::*;
fn main() {
    let network = NetworkDefinition::stokenet();

    let decoder =AddressBech32Decoder::new(&network);

    let from_account_address = ComponentAddress::try_from_bech32(&decoder, "account_tdx_2_12xh47xjynaa57nf4wp9xkvcxaasdle0d9w4gglxuce789dz3tffkzx")
            .expect("Invalid from account address");

    let badge_address = ResourceAddress::try_from_bech32(&decoder, "resource_tdx_2_1ntae2zk8xslaz9hvz0qt4stnqm3trchj0pkqw4aatdq6mn4adgfwa5")
            .expect("Invalid badge address");
           
    let component_address = ComponentAddress::try_from_bech32(&decoder, "component_tdx_2_1crn3hpvu29nuvg74a5zhgr87kkqvktrn39k0429c2n4fmrc90vuv24")
           .expect("Invalid component address");

    let manifest = ManifestBuilder::new()
        // Locking fees from the fee payer's account.
        .withdraw_non_fungibles_from_account(from_account_address, badge_address, [NonFungibleLocalId::integer(1)])
        .take_all_from_worktop(badge_address, "badge")
        .call_method_with_name_lookup(
            component_address,
            "cancel_escrow",
            |lookup| (
                lookup.bucket("badge"),
            )
        )
        .deposit_batch(from_account_address);

    dump_manifest_to_file_system(
        manifest.object_names(),
        &manifest.build(),
        "./transaction_manifest",
        Some("cancel_escrow"),
        &network
    ).err();
}
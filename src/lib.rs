use scrypto::prelude::*;

const DEFAULT_ESCROW_ID: u64 = 1;

#[blueprint]
mod escrow {
    struct Escrow {
        requested_resource: EscrowResourceSpecifier,
        offered_resource: Vault,
        requested_resource_vault: Vault,
        escrow_nft: ResourceAddress,
    }

    impl Escrow {

        pub fn instantiate_escrow(
            requested_resource: EscrowResourceSpecifier,
            offered_resource: Bucket
        ) -> (Global<Escrow>, NonFungibleBucket) {
            // Create a new resource for the EscrowBadge NFT and mint it to the caller
            let escrow_nft = ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
                .metadata(metadata!(
                    init {
                        "name" => "Cool radix hackathon escrow badge", locked;
                    }
                ))
                .mint_roles(mint_roles!(
                    minter => rule!(deny_all);
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles!(
                    burner => rule!(allow_all); // TODO - allow only the escrow contract to burn?
                    burner_updater => rule!(deny_all);
                ))
                .mint_initial_supply(vec![(
                    IntegerNonFungibleLocalId::new(DEFAULT_ESCROW_ID),
                    EscrowBadge {
                        offered_resource: offered_resource.resource_address()
                    }
                )]);

            // Create a new vault for the requested resource
            let requested_resource_vault = Vault::new(requested_resource.get_resource_address());

            // Instantiate the Escrow component
            let escrow = Self {
                requested_resource,
                offered_resource: Vault::with_bucket(offered_resource),
                requested_resource_vault,
                escrow_nft: escrow_nft.resource_address(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            (escrow, escrow_nft)
        }

        pub fn exchange(&mut self, bucket_of_resource: Bucket) -> Bucket {
            // Assert that the offered resource has not been withdrawn
            assert!(
                !self.offered_resource.is_empty(),
                "The offered resource has already been withdrawn!"
            );
            assert_eq!(
                bucket_of_resource.resource_address(),
                self.requested_resource_vault.resource_address(),
                "You must exchange the requested resource, invalid resource specified!"
            );
            match &self.requested_resource {
                EscrowResourceSpecifier::Fungible { amount, .. } => {
                    assert_eq!(
                        *amount,
                        bucket_of_resource.amount(),
                        "You must exchange the requested amount!"
                    );
                },
                EscrowResourceSpecifier::NonFungible { non_fungible_local_id, .. } => {
                    let bucket_id = bucket_of_resource.as_non_fungible().non_fungible_local_id();
                    assert_eq!(
                        *non_fungible_local_id,
                        bucket_id,
                        "You must exchange the requested non-fungible ID!"
                    );
                }
            }

            self.requested_resource_vault.put(bucket_of_resource);

            self.offered_resource.take_all()
        }

        pub fn withdraw_resource(&mut self, escrow_nft: NonFungibleBucket) -> Bucket {
            // Assert that the caller is authorized by checking the NFT
            assert_eq!(
                escrow_nft.resource_address(),
                self.escrow_nft,
                "You must provide the correct escrow NFT to withdraw the resource"
            );
            
            assert!(
                !self.requested_resource_vault.is_empty(),
                "The offer has not been accepted yet, you may want to cancel the escrow instead"
            );

            // Burn the escrow NFT to ensure it can't be used again
            escrow_nft.burn();

            self.requested_resource_vault.take_all()
        }

        pub fn cancel_escrow(&mut self, escrow_nft: NonFungibleBucket) -> Bucket {
            // Assert that the caller is authorized by checking the NFT
            assert_eq!(
                escrow_nft.resource_address(),
                self.escrow_nft,
                "You must provide the correct escrow NFT to withdraw the resource"
            );
            assert!(
                !self.offered_resource.is_empty(),
                "The offered resource has already been withdrawn!"
            );
            // Burn the escrow NFT to ensure it can't be used again
            escrow_nft.burn();
            
            self.offered_resource.take_all()
        }
    }
}



// Types //

#[derive(ScryptoSbor, Clone, ManifestSbor)]
pub enum EscrowResourceSpecifier {
    Fungible {
        resource_address: ResourceAddress,
        amount: Decimal
    },
    NonFungible {
        resource_address: ResourceAddress,
        non_fungible_local_id: NonFungibleLocalId
    }
}

impl EscrowResourceSpecifier {

    pub fn get_resource_address(&self) -> ResourceAddress {
        match self {
            Self::Fungible {
                resource_address, ..
            }
            | Self::NonFungible {
                resource_address, ..
            } => *resource_address,
        }
    }
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct EscrowBadge {
    offered_resource: ResourceAddress
}


#[cfg(test)] mod tests;
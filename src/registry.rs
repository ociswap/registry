use common::utils::assert_fee_rate_within_bounds;
use scrypto::prelude::*;

pub const FEE_PROTOCOL_SHARE_MAX: Decimal = dec!(0.25);

/// Asserts the validity of the configuration parameters for the registry.
///
/// # Parameters
/// - `fee_protocol_share`: The percentage of fees allocated to the protocol.
///                         This must not exceed the maximum allowed protocol fee share.
/// - `sync_period`: The duration (in seconds) over which the synchronization of fees occurs.
///                  Must be greater than zero to ensure a valid operational timeframe.
/// - `sync_slots`: The number of slots into which the sync period is divided. This must be a
///                 positive number and cannot exceed the sync period to ensure proper distribution
///                 of synchronization tasks.
///
/// # Panics
/// - Panics if `fee_protocol_share` exceeds the maximum allowed value, ensuring adherence to protocol fee limits.
/// - Panics if `sync_slots` is zero, as it would imply no synchronization slots.
/// - Panics if `sync_period` is zero, which would result in an undefined synchronization timeframe.
/// - Panics if `sync_slots` is greater than `sync_period`, which would be logically inconsistent as slots should fit within the period.
fn assert_config(fee_protocol_share: Decimal, sync_period: u64, sync_slots: u64) {
    assert_fee_rate_within_bounds(
        fee_protocol_share,
        FEE_PROTOCOL_SHARE_MAX,
        "protocol fee share",
    );
    assert!(
        sync_slots > 0,
        "Number of sync slots needs to be greater than zero."
    );
    assert!(
        sync_period > 0,
        "Sync period needs to be greater than zero."
    );
    assert!(
        sync_slots <= sync_period,
        "Number of sync slots need to be less or equal than duration of the sync period."
    );
}

#[blueprint]
mod registry {
    enable_method_auth! {
        methods {
            sync => PUBLIC;
            update_config => restrict_to: [OWNER];
            withdraw_protocol_fees => restrict_to: [OWNER];
        }
    }
    pub struct Registry {
        owner_badge_address: ResourceAddress,
        protocol_fees: KeyValueStore<ResourceAddress, Vault>,
        fee_protocol_share: Decimal,
        sync_period: u64,
        sync_slots: u64,
    }

    impl Registry {
        /// Create a Registry that will set and collect the protocol fees for the pools.
        ///
        /// # Arguments
        /// * `owner_badge_address`: Owner's badge address that allow you to collect the gathered protocol fees,
        ///  as well as changing parameters in the Registry.
        /// * `fee_protocol_share`: The fraction of the collected fees in the pools that are reserved as revenue for the protocol.
        /// * `sync_period`: States how often the pools should try to send the collected protocol fees to the registry in seconds.
        /// * `sync_slots`: How many slots will be used for the fee collection over time.
        ///    Individual pools use an offset based on their addreess
        ///
        /// # Returns
        /// Returns a Global of the Registry created.
        pub fn instantiate(
            owner_badge_address: ResourceAddress,
            fee_protocol_share: Decimal,
            sync_period: u64,
            sync_slots: u64,
        ) -> Global<Registry> {
            assert_config(fee_protocol_share, sync_period, sync_slots);
            (Self {
                owner_badge_address,
                protocol_fees: KeyValueStore::new(),
                fee_protocol_share,
                sync_period,
                sync_slots,
            })
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(owner_badge_address))))
            .globalize()
        }

        /// Called by the pools, allows depositing the collected protocol fees in the Registry,
        ///  as well as retrieving the currently set fee share and sync parameters.
        ///
        /// # Arguments
        /// * `pool_address`: Address of the Pool calling sync.
        /// * `a_bucket`: Bucket with a protocol fees tokens.
        /// * `b_bucket`: Bucket with b protocol fees tokens.
        ///
        /// # Returns
        /// Returns a tuple consisting of:
        /// * Global of the Pool created.
        /// * Current protocol fee share.
        /// * Next sync time for the Pool.
        pub fn sync(
            &mut self,
            pool_address: ComponentAddress,
            a_bucket: Bucket,
            b_bucket: Bucket,
        ) -> (Decimal, u64) {
            self.put_protocol_fees(a_bucket);
            self.put_protocol_fees(b_bucket);
            (self.fee_protocol_share, self.next_sync_time(pool_address))
        }

        /// Allows the owner to update the configuration of the Registry.
        ///
        /// # Arguments
        /// * `fee_protocol_share`: The fraction of the collected fees in the pools that are reserved as revenue for the protocol.
        /// * `sync_period`: States how often the pools should try to send the collected protocol fees to the registry.
        /// * `sync_slots`: Allows adding an offset to the sync period of the pools.
        pub fn update_config(
            &mut self,
            fee_protocol_share: Decimal,
            sync_period: u64,
            sync_slots: u64,
        ) {
            assert_config(fee_protocol_share, sync_period, sync_slots);
            self.fee_protocol_share = fee_protocol_share;
            self.sync_period = sync_period;
            self.sync_slots = sync_slots;
        }

        /// Allows the owner to withdraw the protocol fees of specific tokens.
        ///
        /// # Arguments
        /// * `addresses`: Addresses of the tokens that will be withdrawed.
        ///
        /// # Returns Buckets containing the withdrawed tokens.
        pub fn withdraw_protocol_fees(&mut self, addresses: Vec<ResourceAddress>) -> Vec<Bucket> {
            addresses
                .into_iter()
                .map(|address| {
                    self.protocol_fees
                        .get_mut(&address)
                        .map_or_else(|| Bucket::new(address), |mut vault| vault.take_all())
                })
                .collect()
        }

        // PRIVATE

        /// Returns next sync time for a specific pool.
        ///
        /// One period is typically 2 weeks. Each pool gets a slot in which it tries to sync again.
        /// The next sync time will be between 1 and 2 periods in the future.
        ///
        /// # Arguments
        /// * `pool_address`: Addresses of the Pool we want to check its next sync time.
        ///
        /// # Returns the next sync time of the Pool.
        fn next_sync_time(&self, pool_address: ComponentAddress) -> u64 {
            let period = self.sync_period;
            let slots = self.sync_slots;

            let now = Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch as u64;
            let slot = component_address_to_u64(&pool_address) % slots;

            let nearest_past_global_cycle_time = (now / period) * period;
            let slot_offset = (period / slots) * slot;
            let next_sync_time = nearest_past_global_cycle_time + slot_offset + period;

            let time_to_next_sync = next_sync_time - now;
            if time_to_next_sync < period {
                return next_sync_time + period;
            }

            next_sync_time
        }

        fn put_protocol_fees(&mut self, bucket: Bucket) {
            if self.protocol_fees.get(&bucket.resource_address()).is_none() {
                self.protocol_fees
                    .insert(bucket.resource_address(), Vault::with_bucket(bucket));
                return;
            }
            self.protocol_fees
                .get_mut(&bucket.resource_address())
                .unwrap()
                .put(bucket);
        }
    }
}

pub fn component_address_to_u64(address: &ComponentAddress) -> u64 {
    let bytes = address.to_vec();
    ((bytes[0] as u64) << 56)
        | ((bytes[1] as u64) << 48)
        | ((bytes[2] as u64) << 40)
        | ((bytes[3] as u64) << 32)
        | ((bytes[4] as u64) << 24)
        | ((bytes[5] as u64) << 16)
        | ((bytes[6] as u64) << 8)
        | (bytes[7] as u64)
}

#[cfg(test)]
mod concentrated_pool_registry {
    use common::math::*;
    use common::pools::SwapType;
    use pretty_assertions::assert_eq;
    use radix_engine::system::system_modules::execution_trace::ResourceSpecifier::Amount;
    use registry::registry::FEE_PROTOCOL_SHARE_MAX;
    use scrypto::prelude::*;
    use scrypto_testenv::*;
    use std::mem;
    use test_case::test_case;
    use registry_test_helper::*;

    #[test]
    fn test_fee_protocol_share_max() {
        assert_eq!(dec!(0.25), FEE_PROTOCOL_SHARE_MAX);
    }

    #[test]
    fn test_instantiate_execute() {
        let mut helper = RegistryTestHelper::new();
        let receipt = helper.instantiate_default(helper.admin_badge_address());
        let registry_address: ComponentAddress = receipt.outputs("instantiate")[0];

        println!("{:?}", receipt.execution_receipt);

        let new_component_address: ComponentAddress = receipt
            .execution_receipt
            .expect_commit_success()
            .new_component_addresses()[0];
        assert_eq!(registry_address, new_component_address);
    }

    #[test_case(-Decimal::ATTO, false ; "negative")]
    #[test_case(dec!(0), true ; "zero")]
    #[test_case(FEE_PROTOCOL_SHARE_MAX/2, true ; "mid")]
    #[test_case(FEE_PROTOCOL_SHARE_MAX, true ; "max")]
    #[test_case(FEE_PROTOCOL_SHARE_MAX + Decimal::ATTO, false ; "higher_than_max")]
    fn test_instantiate_fee_protocol_share_bounds(
        fee_protocol_share: Decimal,
        expect_success: bool,
    ) {
        let mut helper = RegistryTestHelper::new();
        helper.instantiate(helper.admin_badge_address(), fee_protocol_share, 3041, 32);
        if expect_success {
            helper.execute_expect_success(false);
        } else {
            helper.execute_expect_failure(false);
        }
    }

    #[test_case(0, 0, false ; "both_zero")]
    #[test_case(0, 1, false ; "period_zero")]
    #[test_case(1, 0, false ; "slots_zero")]
    #[test_case(1, 1, true ; "period_equal_slots")]
    #[test_case(1, 2, false ; "period_lesser_slots")]
    #[test_case(2, 1, true ; "period_greater_slots")]
    fn test_instantiate_sync_period_slots(sync_period: u64, sync_slots: u64, expect_success: bool) {
        let mut helper = RegistryTestHelper::new();
        helper.instantiate(
            helper.admin_badge_address(),
            dec!(0.1),
            sync_period,
            sync_slots,
        );
        if expect_success {
            helper.execute_expect_success(false);
        } else {
            helper.execute_expect_failure(false);
        }
    }

    #[test_case(0, 0, false ; "both_zero")]
    #[test_case(0, 1, false ; "period_zero")]
    #[test_case(1, 0, false ; "slots_zero")]
    #[test_case(1, 1, true ; "period_equal_slots")]
    #[test_case(1, 2, false ; "period_lesser_slots")]
    #[test_case(2, 1, true ; "period_greater_slots")]
    fn test_update_config_period_and_slots(
        sync_period: u64,
        sync_slots: u64,
        expect_success: bool,
    ) {
        let mut helper = RegistryTestHelper::new();
        helper.instantiate_execute(helper.admin_badge_address(), dec!(0.1), 1, 1);
        helper.load_owner_auth();
        helper.update_config(dec!(0.1), sync_period, sync_slots);
        if expect_success {
            helper.execute_expect_success(false);
        } else {
            helper.execute_expect_failure(false);
        }
    }

    /*#[test]
    fn test_sync_registry_config() {
        let fee_protocol_share = dec!("0.0067");
        let sync_period: u64 = 3041;
        let sync_slots: u64 = 32;

        let mut helper = PoolTestHelper::new_without_instantiate_registry();
        helper.registry.instantiate_execute(
            helper.registry.admin_badge_address(),
            fee_protocol_share,
            sync_period,
            sync_slots,
        );
        // Instantiate pool
        helper.instantiate_default(pdec!(1), false);

        helper.registry.sync(
            helper.pool_address.unwrap(),
            helper.x_address(),
            dec!("1.23"),
            helper.y_address(),
            dec!("1.24"),
        );
        let receipt = helper.registry.execute_expect_success(false);
        let output: Vec<(Decimal, u64)> = receipt.outputs("sync");

        assert_eq!(output, vec![(fee_protocol_share, 5321)]);
    }*/

    #[test]
    fn test_sync_update_config_unauthorized() {
        let mut helper = RegistryTestHelper::new();
        helper.instantiate_execute(helper.admin_badge_address(), dec!("0.1"), 1, 1);
        helper.update_config(dec!("0.1"), 1, 1);
        helper.execute_expect_failure(false);
    }

    /*#[test]
    fn test_sync_update_config() {
        let mut helper = PoolTestHelper::new_without_instantiate_registry();
        helper.registry.instantiate_execute(
            helper.registry.admin_badge_address(),
            dec!("0.1"),
            1,
            1,
        );
        helper.instantiate_default(pdec!(1), false);
        helper.registry.load_owner_auth();
        helper.registry.update_config(dec!(0.2), 100, 42);
        helper.registry.sync(
            helper.pool_address.unwrap(),
            helper.x_address(),
            dec!("1"),
            helper.y_address(),
            dec!("1"),
        );
        let receipt = helper.registry.execute_expect_success(false);
        let output: Vec<(Decimal, u64)> = receipt.outputs("sync");

        assert_eq!(output, vec![(dec!(0.2), 180)]);
    }*/

    #[test]
    fn test_sync_withdraw_protocol_fees_unauthorized() {
        let mut helper = RegistryTestHelper::new();
        helper.instantiate_default(helper.admin_badge_address());
        helper.withdraw_protocol_fees(vec![helper.x_address()]);
        helper.execute_expect_failure(false);
    }

    /*#[test]
    fn test_withdraw_protocol_fees_single_token() {
        let mut helper = PoolTestHelper::new();
        helper.instantiate_default(pdec!(1), false);
        helper.registry.sync(
            helper.pool_address.unwrap(),
            helper.x_address(),
            dec!("1"),
            helper.y_address(),
            dec!("2"),
        );

        helper.registry.load_owner_auth();
        helper
            .registry
            .withdraw_protocol_fees(vec![helper.x_address()]);

        let receipt = helper.registry.execute_expect_success(false);
        let output_buckets = receipt.output_buckets("withdraw_protocol_fees");

        assert_eq!(
            output_buckets,
            vec![vec![Amount(helper.x_address(), dec!(1))]]
        );
    }*/

    /*#[test]
    fn test_withdraw_protocol_fees_single_pool() {
        let mut helper = PoolTestHelper::new_without_instantiate_registry();
        helper.registry.instantiate_execute(
            helper.registry.admin_badge_address(),
            dec!("0.1"),
            1,
            1,
        );

        helper.instantiate_default(pdec!(1), false);
        helper.registry.sync(
            helper.pool_address.unwrap(),
            helper.x_address(),
            dec!("1"),
            helper.y_address(),
            dec!("2"),
        );

        helper.registry.load_owner_auth();
        helper
            .registry
            .withdraw_protocol_fees(vec![helper.x_address(), helper.y_address()]);

        let receipt = helper.registry.execute_expect_success(false);
        let output_buckets = receipt.output_buckets("withdraw_protocol_fees");

        assert_eq!(
            output_buckets,
            vec![vec![
                Amount(helper.x_address(), dec!(1)),
                Amount(helper.y_address(), dec!(2))
            ]]
        );
    }*/

    /*#[test]
    fn test_withdraw_protocol_fees_multiple_pool() {
        let mut helper = PoolTestHelper::new_without_instantiate_registry();
        helper.registry.instantiate_execute(
            helper.registry.admin_badge_address(),
            dec!("0.1"),
            1,
            1,
        );

        helper.instantiate_default(pdec!(1), false);
        helper.registry.sync(
            helper.pool_address.unwrap(),
            helper.x_address(),
            dec!("1"),
            helper.y_address(),
            dec!("2"),
        );
        helper.instantiate_default(pdec!(1), false);
        helper.registry.sync(
            helper.pool_address.unwrap(),
            helper.x_address(),
            dec!("3"),
            helper.y_address(),
            dec!("4"),
        );

        helper.registry.load_owner_auth();
        helper
            .registry
            .withdraw_protocol_fees(vec![helper.x_address(), helper.y_address()]);

        let receipt = helper.registry.execute_expect_success(false);
        let output_buckets = receipt.output_buckets("withdraw_protocol_fees");

        assert_eq!(
            output_buckets,
            vec![vec![
                Amount(helper.x_address(), dec!(4)),
                Amount(helper.y_address(), dec!(6))
            ]]
        );
    }*/

    #[test]
    fn test_update_owner_unauthorized() {
        let mut helper = RegistryTestHelper::new();
        helper.instantiate_default(helper.admin_badge_address());
        helper.set_owner_role(helper.x_address());
        helper.execute_expect_failure(false);
    }

    #[test]
    fn test_update_owner_success() {
        let mut helper = RegistryTestHelper::new();
        helper.instantiate_default(helper.admin_badge_address());
        helper.load_owner_auth();
        helper.set_owner_role(helper.x_address());
        helper.execute_expect_success(false);
    }

    #[test]
    fn test_update_owner_success_twice() {
        let mut helper = RegistryTestHelper::new();
        let account_component = helper.env.account;
        let x_address = helper.x_address();
        let y_address = helper.y_address();

        helper.instantiate_default(helper.admin_badge_address());
        helper.load_owner_auth();
        helper.set_owner_role(helper.x_address());
        helper.execute_expect_success(false);

        let manifest_builder = mem::take(&mut helper.env().manifest_builder);
        helper.env.manifest_builder = manifest_builder.create_proof_from_account_of_amount(
            account_component,
            x_address,
            dec!(1),
        );
        helper.set_owner_role(y_address);
        helper.execute_expect_success(false);
    }

    #[test]
    fn test_update_owner_old_auth_failure() {
        let mut helper = RegistryTestHelper::new();
        helper.instantiate_default(helper.admin_badge_address());
        helper.load_owner_auth();
        helper.set_owner_role(helper.x_address());
        helper.set_owner_role(helper.y_address());
        helper.execute_expect_failure(false);
    }

    /*#[test]
    fn test_sync_pool_swap_advance_time() {
        let fee_protocol_share = dec!(0.25);
        let sync_period: u64 = 3041;
        let sync_slots: u64 = 32;

        let mut helper = PoolTestHelper::new_without_instantiate_registry();
        helper.registry.instantiate_execute(
            helper.registry.admin_badge_address(),
            fee_protocol_share,
            sync_period,
            sync_slots,
        );

        // Instantiate pool
        helper.instantiate_default_with_input_fee(pdec!(1), dec!(0.1), false);
        helper.add_liquidity_success(-10000, 10000, dec!(10), dec!(10), dec!(0), dec!(0));

        helper.swap_success(SwapType::BuyX, dec!(3), dec!(2.440716293858554572), dec!(0));
        helper.registry.load_owner_auth();
        helper
            .registry
            .withdraw_protocol_fees_success(dec!(0), dec!(0));

        helper.swap_success(SwapType::BuyX, dec!(2), dec!(1.382386594462737807), dec!(0));
        helper.registry.load_owner_auth();
        helper
            .registry
            .withdraw_protocol_fees_success(dec!(0), dec!(0));

        helper.swap_success(
            SwapType::SellX,
            dec!(1),
            dec!(1.197018932606031926),
            dec!(0),
        );
        helper.registry.load_owner_auth();
        helper
            .registry
            .withdraw_protocol_fees_success(dec!(0), dec!(0));

        helper.advance_timestamp_by_seconds(5321);

        helper.swap_success(SwapType::BuyX, dec!(4), dec!(2.505473242730227614), dec!(0));
        helper.registry.load_owner_auth();
        helper
            .registry
            .withdraw_protocol_fees_success(dec!(0.025), dec!(0.125));

        helper.advance_timestamp_by_seconds(sync_period);

        helper.swap_success(SwapType::BuyX, dec!(1), dec!(0.541517298529914214), dec!(0));
        helper.registry.load_owner_auth();
        helper
            .registry
            .withdraw_protocol_fees_success(dec!(0), dec!(0.1));

        helper.advance_timestamp_by_seconds(sync_period + 1);

        helper.swap_success(SwapType::BuyX, dec!(1), dec!(0.512948643029352728), dec!(0));
        helper.registry.load_owner_auth();
        helper
            .registry
            .withdraw_protocol_fees_success(dec!(0), dec!(0.025));

        helper.advance_timestamp_by_seconds(sync_period); // still below next sync time due to ceiling next slot time

        helper.swap_success(SwapType::BuyX, dec!(1), dec!(0.486582665710678144), dec!(0));
        helper.swap_success(
            SwapType::SellX,
            dec!(3),
            dec!(4.471281200626990123),
            dec!(0),
        );
        helper.registry.load_owner_auth();
        helper
            .registry
            .withdraw_protocol_fees_success(dec!(0), dec!(0));

        helper.advance_timestamp_by_seconds(sync_period); // we now cross the next sync time and can collect protocol fees
        helper.swap_success(SwapType::BuyX, dec!(1), dec!(0.605186207582635607), dec!(0));
        helper.registry.load_owner_auth();
        helper
            .registry
            .withdraw_protocol_fees_success(dec!(0.075), dec!(0.05));
    }*/
}

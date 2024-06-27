#[cfg(test)]
mod registry {
    use common::math::*;
    use pretty_assertions::assert_eq;
    use registry::registry::FEE_PROTOCOL_SHARE_MAX;
    use registry_test_helper::*;
    use scrypto::prelude::*;
    use scrypto_testenv::*;
    use std::mem;
    use test_case::test_case;

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

    #[test]
    fn test_sync_update_config_unauthorized() {
        let mut helper = RegistryTestHelper::new();
        helper.instantiate_execute(helper.admin_badge_address(), dec!("0.1"), 1, 1);
        helper.update_config(dec!("0.1"), 1, 1);
        helper.execute_expect_failure(false);
    }

    #[test]
    fn test_sync_withdraw_protocol_fees_unauthorized() {
        let mut helper = RegistryTestHelper::new();
        helper.instantiate_default(helper.admin_badge_address());
        helper.withdraw_protocol_fees(vec![helper.x_address()]);
        helper.execute_expect_failure(false);
    }

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
}

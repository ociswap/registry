use pretty_assertions::assert_eq;
use radix_engine::system::system_modules::execution_trace::ResourceSpecifier::Amount;
use scrypto::prelude::*;
use scrypto_testenv::*;
use std::mem;

pub struct RegistryTestHelper {
    pub env: TestEnvironment,
    pub registry_address: Option<ComponentAddress>,
}

impl TestHelperExecution for RegistryTestHelper {
    fn env(&mut self) -> &mut TestEnvironment {
        &mut self.env
    }
}

impl RegistryTestHelper {
    pub fn new() -> Self {
        let packages: HashMap<&str, &str> = vec![("registry", ".")].into_iter().collect();
        println!("{:?}", packages);
        Self::new_with_packages(packages)
    }

    pub fn new_with_packages(packages: HashMap<&str, &str>) -> Self {
        let environment = TestEnvironment::new(packages);
        Self {
            env: environment,
            registry_address: None,
        }
    }

    pub fn instantiate(
        //uses fixed price
        &mut self,
        admin_badge_address: ResourceAddress,
        fee_protocol_rate: Decimal,
        sync_period: u64,
        sync_slots: u64,
    ) -> &mut RegistryTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);
        self.env.manifest_builder = manifest_builder.call_function(
            self.env.package_address("registry"),
            "Registry",
            "instantiate",
            manifest_args!(
                admin_badge_address,
                fee_protocol_rate,
                sync_period,
                sync_slots
            ),
        );

        self.env.new_instruction("instantiate", 1, 0);
        self
    }

    pub fn instantiate_default(&mut self, admin_badge_address: ResourceAddress) -> Receipt {
        //uses fixed price
        self.instantiate(admin_badge_address, dec!("0.1"), 10080 as u64, 20 as u64);

        let receipt = self.execute_expect_success(false);
        let registry_address: ComponentAddress =
            receipt.execution_receipt.expect_commit_success().output(1);
        self.registry_address = Some(registry_address);
        receipt
    }

    pub fn instantiate_execute(
        &mut self,
        admin_badge_address: ResourceAddress,
        fee_protocol_rate: Decimal,
        sync_period: u64,
        sync_slots: u64,
    ) -> Receipt {
        self.instantiate(
            admin_badge_address,
            fee_protocol_rate,
            sync_period,
            sync_slots,
        );

        let receipt = self.execute_expect_success(false);
        let registry_address: ComponentAddress =
            receipt.execution_receipt.expect_commit_success().output(1);
        self.registry_address = Some(registry_address);
        receipt
    }

    pub fn sync(
        &mut self,
        pool_address: ComponentAddress,
        x_address: ResourceAddress,
        x_amount: Decimal,
        y_address: ResourceAddress,
        y_amount: Decimal,
    ) -> &mut RegistryTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        let account_component = self.env.account;

        self.env.manifest_builder = manifest_builder
            .withdraw_from_account(account_component, x_address, x_amount)
            .withdraw_from_account(account_component, y_address, y_amount)
            .take_from_worktop(x_address, x_amount, self.name("x_bucket"))
            .take_from_worktop(y_address, y_amount, self.name("y_bucket"))
            .with_name_lookup(|builder, lookup| {
                let x_bucket = lookup.bucket(self.name("x_bucket"));
                let y_bucket = lookup.bucket(self.name("y_bucket"));
                builder.call_method(
                    self.registry_address.unwrap(),
                    "sync",
                    manifest_args!(pool_address, x_bucket, y_bucket),
                )
            });
        self.env.new_instruction("sync", 5, 4);
        self
    }

    pub fn load_owner_auth(&mut self) -> &mut RegistryTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.create_proof_from_account_of_amount(
            self.env().account,
            self.admin_badge_address(),
            dec!(1),
        );
        self.env.new_instruction("load_owner_auth", 1, 0);
        self
    }

    pub fn update_config(
        &mut self,
        fee_protocol_share: Decimal,
        sync_period: u64,
        sync_slots: u64,
    ) -> &mut RegistryTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.call_method(
            self.registry_address.unwrap(),
            "update_config",
            manifest_args!(fee_protocol_share, sync_period, sync_slots),
        );

        self.env.new_instruction("update_config", 1, 0);
        self
    }

    pub fn withdraw_protocol_fees(
        &mut self,
        addresses: Vec<ResourceAddress>,
    ) -> &mut RegistryTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);
        self.env().manifest_builder = manifest_builder.call_method(
            self.registry_address.unwrap(),
            "withdraw_protocol_fees",
            manifest_args!(addresses),
        );
        self.env.new_instruction("withdraw_protocol_fees", 1, 0);
        self
    }

    pub fn set_owner_role(
        &mut self,
        new_owner_badge_address: ResourceAddress,
    ) -> &mut RegistryTestHelper {
        let manifest_builder = mem::take(&mut self.env.manifest_builder);

        self.env().manifest_builder = manifest_builder.set_owner_role(
            self.registry_address.unwrap(),
            rule!(require(new_owner_badge_address)),
        );
        self.env.new_instruction("set_owner_role", 1, 0);
        self
    }

    pub fn a_address(&self) -> ResourceAddress {
        self.env.a_address
    }

    pub fn b_address(&self) -> ResourceAddress {
        self.env.b_address
    }

    pub fn x_address(&self) -> ResourceAddress {
        self.env.x_address
    }

    pub fn y_address(&self) -> ResourceAddress {
        self.env.y_address
    }

    pub fn v_address(&self) -> ResourceAddress {
        self.env.v_address
    }

    pub fn u_address(&self) -> ResourceAddress {
        self.env.u_address
    }

    pub fn admin_badge_address(&self) -> ResourceAddress {
        self.env.admin_badge_address
    }

    pub fn withdraw_protocol_fees_success(
        &mut self,
        x_amount_expected: Decimal,
        y_amount_expected: Decimal,
    ) {
        self.withdraw_protocol_fees(vec![self.x_address(), self.y_address()]);
        let receipt = self.execute_expect_success(false);

        let output_buckets = receipt.output_buckets("withdraw_protocol_fees");

        assert_eq!(
            output_buckets,
            vec![vec![
                Amount(self.x_address(), x_amount_expected),
                Amount(self.y_address(), y_amount_expected)
            ]]
        );
    }
}

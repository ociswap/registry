#[cfg(test)]
mod registry_manifest_templates {
    use std::mem;

    // INSTANTIATE
    use scrypto::prelude::*;
    use scrypto_unit::dump_manifest_to_file_system;
    use registry_test_helper::*;

    #[test]
    fn test_dump_instantiate() {
        let mut helper: RegistryTestHelper = RegistryTestHelper::new();
        helper.instantiate(
            helper.admin_badge_address(),
            dec!("0.1"),
            10080 as u64,
            20 as u64,
        );
        let manifest_builder =
            mem::take(&mut helper.env.manifest_builder).deposit_batch(helper.env.account);
        dump_manifest_to_file_system(
            manifest_builder.object_names(),
            &manifest_builder.build(),
            "./transaction-manifest",
            Some("instantiate"),
            &NetworkDefinition::simulator(),
        )
        .err();
    }
}

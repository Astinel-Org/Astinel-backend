use sentinel_core::Rule;

mod auth_mistake;
mod contract_upgrade;
mod dead_code;
mod gas_optimization;
mod integer_overflow;
mod large_storage_write;
mod missing_require_auth;
mod missing_ttl;
mod unsafe_panic;
mod unused_storage;

pub fn register_all() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(missing_require_auth::MissingRequireAuth),
        Box::new(unsafe_panic::UnsafePanic),
        Box::new(large_storage_write::LargeStorageWrite),
        Box::new(dead_code::DeadCode),
        Box::new(unused_storage::UnusedStorage),
        Box::new(missing_ttl::MissingTtl),
        Box::new(auth_mistake::AuthMistake),
        Box::new(integer_overflow::IntegerOverflow),
        Box::new(gas_optimization::GasOptimization),
        Box::new(contract_upgrade::ContractUpgrade),
    ]
}

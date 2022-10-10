use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub type ThreadSafeUsageTime = Arc<RwLock<HashMap<String, u64>>>;
pub type UsageTime = HashMap<String, u64>;

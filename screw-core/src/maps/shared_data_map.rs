use crate::maps::DataMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SharedDataMap(Arc<DataMap>);

impl SharedDataMap {
    pub fn new(data_map: DataMap) -> Self {
        Self(Arc::new(data_map))
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.0.get::<T>()
    }
}

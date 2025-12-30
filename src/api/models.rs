use crate::api::metadata::MetadataStore;

#[derive(Clone)]
pub struct ApiState {
    pub metadata: MetadataStore,
}
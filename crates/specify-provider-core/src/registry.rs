use std::collections::HashMap;
use std::sync::Arc;

use crate::contract::Provider;
use crate::errors::ProviderError;
use crate::models::ProviderKind;

pub struct ProviderRegistry {
    providers: HashMap<ProviderKind, Arc<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register<P: Provider + 'static>(&mut self, provider: P) {
        self.providers.insert(provider.kind(), Arc::new(provider));
    }

    pub fn get(&self, kind: &ProviderKind) -> Option<Arc<dyn Provider>> {
        self.providers.get(kind).cloned()
    }

    pub fn require(&self, kind: &ProviderKind) -> Result<Arc<dyn Provider>, ProviderError> {
        self.get(kind).ok_or_else(|| {
            ProviderError::Unsupported(format!("provider {:?} is not registered", kind))
        })
    }

    pub fn activate(&self, kind: &ProviderKind) -> Result<Arc<dyn Provider>, ProviderError> {
        let provider = self.require(kind)?;
        let capabilities = provider.capabilities();
        if !capabilities.supports_contract_baseline() {
            return Err(ProviderError::Conformance(format!(
                "provider {:?} does not meet baseline capabilities",
                kind
            )));
        }
        provider.health_check()?;
        Ok(provider)
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

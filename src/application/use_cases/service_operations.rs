use crate::domain::{
    entities::Service,
    repositories::ServiceRepository,
};
use anyhow::Result;
use std::sync::Arc;

pub struct ServiceRepositoryUseCase {
    repository: Arc<dyn ServiceRepository>,
}

impl ServiceRepositoryUseCase {
    pub fn new(repository: Arc<dyn ServiceRepository>) -> Self {
        Self { repository }
    }

    pub fn repository(&self) -> Arc<dyn ServiceRepository> {
        Arc::clone(&self.repository)
    }
}

pub struct ListServices {
    use_case: ServiceRepositoryUseCase,
}

impl ListServices {
    pub fn new(repository: Arc<dyn ServiceRepository>) -> Self {
        Self {
            use_case: ServiceRepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self) -> Result<Vec<Service>> {
        self.use_case.repository().list_services().await
    }
}

pub struct StartService {
    use_case: ServiceRepositoryUseCase,
}

impl StartService {
    pub fn new(repository: Arc<dyn ServiceRepository>) -> Self {
        Self {
            use_case: ServiceRepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, service_name: &str) -> Result<()> {
        self.use_case.repository().start_service(service_name).await
    }
}

pub struct StopService {
    use_case: ServiceRepositoryUseCase,
}

impl StopService {
    pub fn new(repository: Arc<dyn ServiceRepository>) -> Self {
        Self {
            use_case: ServiceRepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, service_name: &str) -> Result<()> {
        self.use_case.repository().stop_service(service_name).await
    }
}

pub struct RestartService {
    use_case: ServiceRepositoryUseCase,
}

impl RestartService {
    pub fn new(repository: Arc<dyn ServiceRepository>) -> Self {
        Self {
            use_case: ServiceRepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, service_name: &str) -> Result<()> {
        self.use_case.repository().restart_service(service_name).await
    }
}

use std::path::Path;

use runtime_api::permissions::PermissionGuard;
use runtime_sandbox::manifest::AppManifest;
use runtime_sandbox::scope::AppScope;

pub struct AppContext {
    pub permissions: PermissionGuard,
    pub scope: AppScope,
    pub app_name: String,
}

impl AppContext {
    pub fn from_manifest(manifest: AppManifest) -> Self {
        let app_name = manifest.app.name.clone();
        let scope = AppScope::new(&app_name);
        let permissions = PermissionGuard::new(manifest);
        Self {
            permissions,
            scope,
            app_name,
        }
    }

    pub fn from_manifest_file(path: &Path) -> Result<Self, runtime_sandbox::manifest::ManifestError> {
        let manifest = AppManifest::from_file(path)?;
        Ok(Self::from_manifest(manifest))
    }

    pub fn data_dir(&self) -> &Path {
        self.scope.data_dir()
    }

    pub fn config_dir(&self) -> &Path {
        self.scope.config_dir()
    }

    pub fn cache_dir(&self) -> &Path {
        self.scope.cache_dir()
    }
}

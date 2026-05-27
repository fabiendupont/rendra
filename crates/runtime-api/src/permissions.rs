use runtime_sandbox::manifest::AppManifest;

#[derive(Debug, thiserror::Error)]
#[error("permission denied: {feature} is not declared in app.toml")]
pub struct PermissionDenied {
    pub feature: String,
}

pub struct PermissionGuard {
    manifest: AppManifest,
}

impl PermissionGuard {
    pub fn new(manifest: AppManifest) -> Self {
        Self { manifest }
    }

    pub fn check(&self, feature: &str) -> Result<(), PermissionDenied> {
        if self.manifest.has_permission(feature) {
            Ok(())
        } else {
            Err(PermissionDenied {
                feature: feature.to_string(),
            })
        }
    }

    pub fn manifest(&self) -> &AppManifest {
        &self.manifest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_guard(toml_str: &str) -> PermissionGuard {
        let manifest: AppManifest = toml::from_str(toml_str).unwrap();
        PermissionGuard::new(manifest)
    }

    #[test]
    fn allows_declared_permission() {
        let guard = make_guard(r#"
[app]
name = "test"
version = "0.1.0"

[permissions]
network = ["https"]
"#);
        assert!(guard.check("network").is_ok());
    }

    #[test]
    fn denies_undeclared_permission() {
        let guard = make_guard(r#"
[app]
name = "test"
version = "0.1.0"
"#);
        assert!(guard.check("network").is_err());
        assert!(guard.check("filesystem").is_err());
        assert!(guard.check("clipboard").is_err());
    }

    #[test]
    fn denies_unknown_feature() {
        let guard = make_guard(r#"
[app]
name = "test"
version = "0.1.0"

[permissions]
network = ["https"]
"#);
        assert!(guard.check("nuclear-launch").is_err());
    }
}

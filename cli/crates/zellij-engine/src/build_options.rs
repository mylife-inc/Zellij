use zellij_core::{build_overrides::BuildOverrides, features::Features};

#[derive(Clone, Debug, Default)]
pub struct BuildOptions {
    pub features: Option<Features>,
    pub overrides: BuildOverrides,
    pub build_args: Vec<zellij_core::build_args::BuildArg>,
}

impl BuildOptions {
    pub fn require_no_image_args(&self) -> zellij_core::Result<()> {
        if !self.build_args.is_empty() {
            return Err(zellij_core::Error::InvalidArgument(
                "--build-arg requires an image build or image publication; static-folder builds and Netlify do not build images".into(),
            ));
        }
        Ok(())
    }

    pub fn with_features(features: Option<Features>) -> Self {
        Self {
            features,
            ..Self::default()
        }
    }

    pub(crate) fn environment(&self) -> Vec<String> {
        self.overrides
            .encoded()
            .map(|value| vec!["-e".into(), format!("ZELLIJ_BUILD_OVERRIDES={value}")])
            .unwrap_or_default()
    }

    pub(crate) fn docker_environment(&self) -> String {
        self.overrides
            .encoded()
            .map(|value| format!("ENV ZELLIJ_BUILD_OVERRIDES={value}\n"))
            .unwrap_or_default()
    }

    pub(crate) fn guard(&self) -> &'static str {
        if self.overrides.is_empty() {
            return "";
        }
        "node -e 'if(require(\"/opt/zellij/package.json\").zellij?.buildOverridesVersion!==1){console.error(\"This engine does not support build overrides; choose a current ZELLIJ_IMAGE.\");process.exit(2)}' || exit 95; "
    }

    pub(crate) fn docker_guard(&self) -> String {
        let guard = self.guard();
        if guard.is_empty() {
            String::new()
        } else {
            format!("RUN {}\n", guard.trim_end())
        }
    }

    pub(crate) fn docker_check(&self) -> &'static str {
        if self.overrides.no_check {
            "# Separate preflight skipped; preparation and rendering still validate content.\n"
        } else {
            "RUN npx zellij check content\n"
        }
    }
}

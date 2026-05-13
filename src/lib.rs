use zed_extension_api::{self as zed, LanguageServerId, Result};

struct PythonAutodocExtension {
    cached_binary_path: Option<String>,
}

impl zed::Extension for PythonAutodocExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // Look for the pre-built LSP binary that ships alongside this extension.
        // When installed from the registry, Zed extracts the extension and the
        // binary lives next to the WASM file.  During local dev (`Install Dev
        // Extension`) it lives at the path below relative to the workspace root.
        let binary_path = self.lsp_binary_path(worktree)?;

        Ok(zed::Command {
            command: binary_path,
            args: vec![],
            env: vec![],
        })
    }
}

impl PythonAutodocExtension {
    fn lsp_binary_path(&mut self, worktree: &zed::Worktree) -> Result<String> {
        // Return the cached path if we already resolved it this session.
        if let Some(path) = &self.cached_binary_path {
            if std::path::Path::new(path).exists() {
                return Ok(path.clone());
            }
        }

        let binary_name = if cfg!(target_os = "windows") {
            "python-autodoc-lsp.exe"
        } else {
            "python-autodoc-lsp"
        };

        // First, try to find the binary in the project's target directory (for dev extensions)
        let target_paths = [
            format!("target/x86_64-unknown-linux-gnu/release/{}", binary_name),
            format!("target/release/{}", binary_name),
        ];

        for rel_path in &target_paths {
            let full_path = format!("{}/{}", worktree.root_path(), rel_path);
            if std::path::Path::new(&full_path).exists() {
                self.cached_binary_path = Some(full_path.clone());
                return Ok(full_path);
            }
        }

        // Fallback: try to find it on PATH
        if let Some(path) = worktree.which(binary_name) {
            self.cached_binary_path = Some(path.clone());
            return Ok(path);
        }

        Err(format!(
            "python-autodoc-lsp binary not found. \
             Please build the LSP server with `cargo build --release --target x86_64-unknown-linux-gnu` \
             from the crates/lsp-server directory."
        ))
    }
}

zed::register_extension!(PythonAutodocExtension);

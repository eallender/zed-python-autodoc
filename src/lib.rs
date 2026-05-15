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
        if let Some(path) = &self.cached_binary_path {
            if std::path::Path::new(path).exists() {
                return Ok(path.clone());
            }
        }

        let (os, arch) = zed::current_platform();

        let binary_name = match os {
            zed::Os::Windows => "python-autodoc-lsp.exe",
            _ => "python-autodoc-lsp",
        };

        let target_triple = match (os, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "aarch64-apple-darwin",
            (zed::Os::Mac, zed::Architecture::X8664) => "x86_64-apple-darwin",
            (zed::Os::Mac, zed::Architecture::X86) => "i686-apple-darwin",
            (zed::Os::Linux, zed::Architecture::Aarch64) => "aarch64-unknown-linux-gnu",
            (zed::Os::Linux, zed::Architecture::X8664) => "x86_64-unknown-linux-gnu",
            (zed::Os::Linux, zed::Architecture::X86) => "i686-unknown-linux-gnu",
            (zed::Os::Windows, zed::Architecture::Aarch64) => "aarch64-pc-windows-msvc",
            (zed::Os::Windows, zed::Architecture::X8664) => "x86_64-pc-windows-msvc",
            (zed::Os::Windows, zed::Architecture::X86) => "i686-pc-windows-msvc",
        };

        let target_paths = [
            format!("target/{}/release/{}", target_triple, binary_name),
            format!("target/release/{}", binary_name),
        ];

        for rel_path in &target_paths {
            let full_path = format!("{}/{}", worktree.root_path(), rel_path);
            eprintln!("[python-autodoc] checking: {}", full_path);
            if std::path::Path::new(&full_path).exists() {
                eprintln!("[python-autodoc] found at: {}", full_path);
                self.cached_binary_path = Some(full_path.clone());
                return Ok(full_path);
            }
        }

        eprintln!("[python-autodoc] trying PATH lookup for '{}'", binary_name);
        if let Some(path) = worktree.which(binary_name) {
            eprintln!("[python-autodoc] found via PATH: {}", path);
            self.cached_binary_path = Some(path.clone());
            return Ok(path);
        }

        let err = format!(
            "python-autodoc-lsp binary not found. \
             Build it with `cargo build --release --target {}` \
             from the crates/lsp-server directory, or add python-autodoc-lsp to your PATH.",
            target_triple
        );
        eprintln!("[python-autodoc] ERROR: {}", err);
        Err(err)
    }
}

zed::register_extension!(PythonAutodocExtension);

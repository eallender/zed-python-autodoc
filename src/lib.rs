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
            (zed::Os::Linux, zed::Architecture::Aarch64) => "aarch64-unknown-linux-gnu",
            (zed::Os::Linux, zed::Architecture::X8664) => "x86_64-unknown-linux-gnu",
            (zed::Os::Linux, zed::Architecture::X86) => "i686-unknown-linux-gnu",
            (zed::Os::Windows, zed::Architecture::Aarch64) => "aarch64-pc-windows-msvc",
            (zed::Os::Windows, zed::Architecture::X8664) => "x86_64-pc-windows-msvc",
            (zed::Os::Windows, zed::Architecture::X86) => "i686-pc-windows-msvc",
            _ => return Err("python-autodoc-lsp does not support this platform".to_string()),
        };

        // Download the pre-built binary from GitHub Releases.
        // Release assets must follow the naming convention:
        //   python-autodoc-lsp-{target_triple}.tar.gz
        // containing a single `python-autodoc-lsp[.exe]` binary at the archive root.
        let asset = format!("python-autodoc-lsp-{}.tar.gz", target_triple);
        let url = format!(
            "https://github.com/eallender/zed-python-autodoc/releases/download/v{}/{}",
            env!("CARGO_PKG_VERSION"),
            asset
        );
        // download_file extracts the tar into a directory named binary_name,
        // so the actual binary path is binary_name/binary_name.
        let extracted_path = format!("{}/{}", binary_name, binary_name);
        let download_err =
            match zed::download_file(&url, binary_name, zed::DownloadedFileType::GzipTar) {
                Ok(()) => {
                    zed::make_file_executable(&extracted_path)?;
                    self.cached_binary_path = Some(extracted_path.clone());
                    return Ok(extracted_path);
                }
                Err(e) => e,
            };

        // Last resort: binary installed globally on PATH.
        if let Some(path) = worktree.which(binary_name) {
            self.cached_binary_path = Some(path.clone());
            return Ok(path);
        }

        Err(format!(
            "python-autodoc-lsp binary not found. \
             Download from GitHub Releases failed ({download_err}). \
             Build it with `cargo build --release --target {target_triple}` \
             from the crates/lsp-server directory, or add python-autodoc-lsp to your PATH.",
        ))
    }
}

zed::register_extension!(PythonAutodocExtension);

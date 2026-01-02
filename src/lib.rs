use std::fs;
use zed_extension_api::{self as zed, Command, ContextServerId, Project, Result};

const MCPR_REPO: &str = "moshyfawn/mcp-relay";
const BINARY_NAME: &str = "mcp-relay";
const MCP_URL: &str = "https://font.emtech.cc/mcp";

struct EmfontMCPExtension {
    binary_path: Option<String>,
}

impl EmfontMCPExtension {
    fn get_binary(&mut self) -> Result<String> {
        if let Some(ref path) = self.binary_path {
            return Ok(path.clone());
        }

        let (platform, arch) = zed::current_platform();

        let binary_name = if platform == zed::Os::Windows {
            "mcp_relay.exe".to_string()
        } else {
            "mcp_relay".to_string()
        };

        let release = zed::latest_github_release(
            MCPR_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset_name = format!(
            "{}-{}-{}.tar.gz",
            BINARY_NAME,
            match platform {
                zed::Os::Mac => "darwin",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "windows",
            },
            match arch {
                zed::Architecture::Aarch64 => "aarch64",
                zed::Architecture::X8664 | zed::Architecture::X86 => "x86_64",
            }
        );

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("Asset not found: {}", asset_name))?;

        let version_dir = format!("{}-{}", BINARY_NAME, release.version);
        let binary_path = format!("{}/{}", version_dir, binary_name);

        if fs::metadata(&binary_path).is_err() {
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    if name.to_string_lossy().starts_with(BINARY_NAME) {
                        let _ = fs::remove_dir_all(entry.path());
                    }
                }
            }

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::GzipTar,
            )
            .map_err(|e| format!("Download failed: {}", e))?;

            if platform != zed::Os::Windows {
                zed::make_file_executable(&binary_path)?;
            }
        }

        self.binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for EmfontMCPExtension {
    fn new() -> Self {
        Self { binary_path: None }
    }

    fn context_server_command(
        &mut self,
        _id: &ContextServerId,
        _project: &Project,
    ) -> Result<Command> {
        Ok(Command {
            command: self.get_binary()?,
            args: vec![MCP_URL.to_string()],
            env: vec![],
        })
    }
}

zed::register_extension!(EmfontMCPExtension);

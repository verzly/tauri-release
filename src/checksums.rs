use anyhow::Result;
use colored::*;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;
use walkdir::WalkDir;

pub fn generate_checksums(dir: &Path) -> Result<()> {
    println!("{}", "Generating SHA-256 checksums".yellow().bold());

    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) == Some("sha256") {
            continue;
        }

        let filename = path.file_name().and_then(|value| value.to_str()).unwrap_or("artifact");
        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        let hash = hex::encode(hasher.finalize());
        fs::write(path.with_extension(format!("{}sha256", path.extension().and_then(|v| v.to_str()).map(|v| format!("{}.", v)).unwrap_or_default())), format!("{}  {}\n", hash, filename))?;
    }

    Ok(())
}

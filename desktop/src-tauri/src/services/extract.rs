use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub struct ExtractService;

impl ExtractService {
    pub fn extract_7z(
        archive_path: &Path,
        output_dir: &Path,
        password: Option<&str>,
    ) -> Result<()> {
        let _filename = archive_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        crate::logger::log(&format!(
            "[EXTRACT] Extracting archive: {} to {}",
            archive_path.display(),
            output_dir.display()
        ));

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(output_dir)?;

        let start = Instant::now();

        // Use 7z CLI - it handles both single and split archives natively
        let mut cmd = Command::new(crate::services::binary_paths::sevenz());
        cmd.arg("x") // Extract with full paths
            .arg(format!("-o{}", output_dir.display())) // Output directory
            .arg(archive_path) // Archive path (7z handles .001 files automatically)
            .arg("-y"); // Assume yes to all prompts

        // Add password if provided
        if let Some(pw) = password {
            cmd.arg(format!("-p{}", pw));
        } else {
            cmd.arg("-p"); // No password
        }

        crate::logger::log(&format!(
            "[EXTRACT] Running: {:?}",
            cmd.get_args().collect::<Vec<_>>()
        ));

        let output = cmd
            .output()
            .map_err(|e| anyhow!("Failed to execute 7z: {}. Is 7z installed?", e))?;

        let elapsed = start.elapsed();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            crate::logger::log(&format!(
                "[EXTRACT] 7z failed with status: {}",
                output.status
            ));
            crate::logger::log(&format!("[EXTRACT] stderr: {}", stderr));
            crate::logger::log(&format!("[EXTRACT] stdout: {}", stdout));
            return Err(anyhow!(
                "7z extraction failed: {}\nstderr: {}",
                output.status,
                stderr
            ));
        }

        crate::logger::log(&format!("[EXTRACT] Extraction complete in {:?}", elapsed));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_scratch() {
        let scratch_dir = PathBuf::from("../../scratch");
        let archive = scratch_dir.join("c27e4a47afe298c72f7a7b2eb5daf6fe.7z.001");
        if !archive.exists() {
            println!("Scratch archive not found, skipping test");
            return;
        }

        let output = scratch_dir.join("test_output");
        let _ = std::fs::remove_dir_all(&output);
        let _ = std::fs::create_dir_all(&output);

        let password = Some("gL59VfgPxoHR");
        let start = Instant::now();
        let res = ExtractService::extract_7z(&archive, &output, password);
        let elapsed = start.elapsed();

        println!("Single volume extraction completed in {:?}", elapsed);
        assert!(res.is_ok(), "Extraction failed: {:?}", res.err());

        let entries = std::fs::read_dir(&output).unwrap();
        assert!(entries.count() > 0, "No files extracted");
    }

    #[test]
    fn test_extract_multipart() {
        let scratch_dir = PathBuf::from("../../scratch");
        let archive = scratch_dir
            .join("ac3d86d25183ee27b0cc80f7384e427b/ac3d86d25183ee27b0cc80f7384e427b.7z.001");
        if !archive.exists() {
            println!("Multipart archive not found, skipping test");
            return;
        }

        let output = scratch_dir.join("test_multipart_output");
        let _ = std::fs::remove_dir_all(&output);
        let _ = std::fs::create_dir_all(&output);

        println!("Starting multipart extraction test with 7z CLI...");
        let start = Instant::now();

        let password = Some("gL59VfgPxoHR");
        let res = ExtractService::extract_7z(&archive, &output, password);
        let elapsed = start.elapsed();

        println!("Multipart extraction completed in {:?}", elapsed);
        assert!(res.is_ok(), "Multipart extraction failed: {:?}", res.err());

        let entries: Vec<_> = std::fs::read_dir(&output).unwrap().collect();
        println!("Extracted {} files/directories", entries.len());
        assert!(
            !entries.is_empty(),
            "No files extracted from multipart archive"
        );

        // Print extracted file sizes
        for entry in entries {
            let entry = entry.unwrap();
            let metadata = entry.metadata().unwrap();
            println!(
                "  {}: {} bytes",
                entry.file_name().to_string_lossy(),
                metadata.len()
            );
        }
    }
}

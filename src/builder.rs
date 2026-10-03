use std::{fs, process::Command};

pub struct FasmCompiler {
    pub proj_name: String,
    pub is_release: bool,
}

impl FasmCompiler {
    pub fn comp_link(&self, code: &str) -> Result<(), String> {
        let mut tmp_path = std::env::temp_dir();
        tmp_path.push(format!("{}_tmp.asm", self.proj_name));

        fs::write(&tmp_path, code)
            .map_err(|e| format!("[!] Fail: Failed to write tmp assembly file: {}", e))?;

        let output_bin = format!("./{}", self.proj_name);

        let output = Command::new("fasm")
            .arg(&tmp_path)
            .arg(&output_bin)
            .output()
            .map_err(|e| format!("[!] Fail: Failed to exec fasm: {}", e))?;

        let _ = fs::remove_file(&tmp_path);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!("[!] Fail: FASM Error: \n{}\n{}", stdout, stderr));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&output_bin, fs::Permissions::from_mode(0o755));
        }

        println!("[*] Build Successful: {}", output_bin);
        Ok(())
    }
}

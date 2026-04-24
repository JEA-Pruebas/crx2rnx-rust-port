use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::CrxError;
use crate::header::parse_header;

pub fn decompress_crinex_native(input: &str) -> Result<String, CrxError> {
    let _header = parse_header(input)?;
    run_original_crx2rnx(input)
}

fn run_original_crx2rnx(input: &str) -> Result<String, CrxError> {
    let helper = env_helper_path()?;
    let work_dir = create_work_dir()?;

    let input_path = work_dir.join("input.25d");
    let output_path = work_dir.join("input.25o");

    fs::write(&input_path, input)
        .map_err(|e| CrxError::new(format!("No se pudo escribir input temporal: {e}")))?;

    let output = Command::new(&helper)
        .arg(path_to_arg(&input_path)?)
        .current_dir(&work_dir)
        .output()
        .map_err(|e| {
            CrxError::new(format!(
                "No se pudo ejecutar helper CRX2RNX en {}: {e}",
                helper.display()
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = fs::remove_dir_all(&work_dir);
        return Err(CrxError::new(format!(
            "Fallo CRX2RNX helper (status {:?}): {}",
            output.status.code(),
            stderr.trim()
        )));
    }

    let content = fs::read_to_string(&output_path).map_err(|e| {
        CrxError::new(format!(
            "Helper ejecutó pero no se pudo leer salida {}: {e}",
            output_path.display()
        ))
    })?;

    let _ = fs::remove_dir_all(&work_dir);
    Ok(content)
}

fn env_helper_path() -> Result<PathBuf, CrxError> {
    let raw = env!("CRX2RNX_HELPER");
    if raw.is_empty() {
        return Err(CrxError::new(
            "No se compiló el helper C de CRX2RNX (variable CRX2RNX_HELPER vacía)",
        ));
    }

    let path = PathBuf::from(raw);
    if !path.exists() {
        return Err(CrxError::new(format!(
            "No existe helper C de CRX2RNX en {}",
            path.display()
        )));
    }
    Ok(path)
}

fn create_work_dir() -> Result<PathBuf, CrxError> {
    let base = std::env::temp_dir();
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| CrxError::new(format!("Reloj del sistema inválido: {e}")))?
        .as_nanos();

    let dir = base.join(format!("crx2rnx-rust-{pid}-{nanos}"));
    fs::create_dir_all(&dir).map_err(|e| {
        CrxError::new(format!(
            "No se pudo crear directorio temporal {}: {e}",
            dir.display()
        ))
    })?;
    Ok(dir)
}

fn path_to_arg(path: &Path) -> Result<String, CrxError> {
    path.to_str()
        .map(ToString::to_string)
        .ok_or_else(|| CrxError::new("Ruta temporal contiene UTF-8 inválido"))
}
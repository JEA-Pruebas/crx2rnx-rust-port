use crate::error::CrxError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrinexHeader {
    pub crinex_version: String,
    pub rinex_version: Option<String>,
    pub data_start: usize,
}

pub fn parse_header(input: &str) -> Result<CrinexHeader, CrxError> {
    let mut offset = 0usize;
    let mut first_line: Option<&str> = None;
    let mut rinex_version: Option<String> = None;

    for raw_line in input.split_inclusive('\n') {
        let line = raw_line.trim_end_matches(['\r', '\n']);

        if first_line.is_none() {
            first_line = Some(line);
        }

        if line.contains("RINEX VERSION / TYPE") {
            rinex_version = Some(line.chars().take(20).collect::<String>().trim().to_string());
        }

        offset += raw_line.len();
        if line.contains("END OF HEADER") {
            let first = first_line.ok_or_else(|| CrxError::new("Entrada vacía"))?;
            validate_crinex_signature(first)?;

            let crinex_version = first
                .chars()
                .take(20)
                .collect::<String>()
                .trim()
                .to_string();

            if !crinex_version.starts_with("1.0") {
                return Err(CrxError::new(format!(
                    "Versión CRINEX no soportada: {crinex_version} (solo 1.0 por ahora)"
                )));
            }

            return Ok(CrinexHeader {
                crinex_version,
                rinex_version,
                data_start: offset,
            });
        }
    }

    Err(CrxError::new(
        "No se encontró END OF HEADER en la entrada CRINEX",
    ))
}

fn validate_crinex_signature(first_line: &str) -> Result<(), CrxError> {
    if !first_line.contains("COMPACT RINEX FORMAT") {
        return Err(CrxError::new(
            "La primera línea no parece ser un archivo Compact RINEX",
        ));
    }
    if !first_line.contains("CRINEX VERS   / TYPE") {
        return Err(CrxError::new(
            "No se encontró la etiqueta CRINEX VERS   / TYPE en el encabezado",
        ));
    }
    Ok(())
}
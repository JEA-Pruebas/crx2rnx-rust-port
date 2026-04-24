use crx2rnx_port::{PureRustDebugRecord, decompress_crinex_pure, decompress_crinex_pure_debug};

fn normalize_lines(s: &str) -> Vec<String> {
    s.replace("\r\n", "\n")
        .lines()
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
}

#[test]
fn test_pure_diagnostic_line_by_line_summary() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let expected = std::fs::read_to_string("../samples/igm1252a.25o")
        .expect("Debe poder leerse el sample RINEX esperado");

    let output = decompress_crinex_pure(&input).expect("Debe generar salida pura");

    let out_lines = normalize_lines(&output);
    let exp_lines = normalize_lines(&expected);

    let compared_total = out_lines.len().min(exp_lines.len());
    let mut first_diff = None;
    let mut equal_prefix = 0usize;

    for idx in 0..compared_total {
        if out_lines[idx] == exp_lines[idx] {
            equal_prefix += 1;
        } else {
            first_diff = Some(idx);
            break;
        }
    }

    if first_diff.is_none() && out_lines.len() != exp_lines.len() {
        first_diff = Some(compared_total);
    }

    let mut examples = Vec::new();
    for idx in 0..compared_total {
        if out_lines[idx] != exp_lines[idx] {
            examples.push((idx + 1, exp_lines[idx].clone(), out_lines[idx].clone()));
            if examples.len() == 3 {
                break;
            }
        }
    }

    eprintln!("[pure-diff] equal_prefix_lines={equal_prefix}");
    eprintln!("[pure-diff] compared_total_lines={compared_total}");
    eprintln!("[pure-diff] expected_total_lines={}", exp_lines.len());
    eprintln!("[pure-diff] output_total_lines={}", out_lines.len());
    eprintln!(
        "[pure-diff] first_diff_line={:?}",
        first_diff.map(|n| n + 1)
    );
    for (line_no, expected_line, output_line) in examples {
        eprintln!("[pure-diff] line {line_no} expected: {expected_line}");
        eprintln!("[pure-diff] line {line_no} output  : {output_line}");
    }

    // No exigimos paridad total aún; sí exigimos progreso verificable.
    assert!(
        equal_prefix >= 103,
        "Se esperaba mejorar paridad más allá de 102 líneas"
    );
    assert!(
        compared_total > 100,
        "Se esperaba comparar una porción amplia del archivo"
    );
}

#[test]
fn test_pure_diagnostic_first_observation_line_includes_lli_ssi_flags() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let expected = std::fs::read_to_string("../samples/igm1252a.25o")
        .expect("Debe poder leerse el sample RINEX esperado");

    let output = decompress_crinex_pure(&input).expect("Debe generar salida pura");

    let out_lines = normalize_lines(&output);
    let exp_lines = normalize_lines(&expected);

    let idx = exp_lines
        .iter()
        .position(|l| l.contains("119019616.473") && l.contains("92742542.658"))
        .expect("Debe existir primera línea de observaciones en expected");

    let exp_line = &exp_lines[idx];
    let out_line = out_lines
        .get(idx)
        .expect("La salida pura debe tener línea equivalente por posición");

    // En expected hay flags LLI/SSI embebidos en el campo y la salida pura
    // ahora debe comenzar a reflejarlos en el primer bloque.
    assert!(exp_line.contains("119019616.473 6"));
    assert!(out_line.contains("119019616.473 6"));
    assert!(out_line.contains("92742542.658 9"));
}

#[test]
fn test_pure_flag_mapping_specific_cases_g28_y_r07() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let output = decompress_crinex_pure(&input).expect("Debe generar salida pura");
    let out_lines = normalize_lines(&output);

    assert!(
        out_lines.iter().any(|l| {
            l.contains("126242459.499 5")
                && l.contains("98370843.793 9")
                && l.contains("24023160.203 5")
                && l.contains("24023168.33647")
        }),
        "Caso G28: la línea principal debe conservar 24023168.33647"
    );
    assert!(
        out_lines
            .iter()
            .any(|l| l.contains("2366.043 5") && l.contains("1843.810 9")),
        "Caso G28: la línea de continuación debe conservar 1843.810 9"
    );
    assert!(
        out_lines.iter().any(|l| {
            l.contains("114753391.111 4")
                && l.contains("89252657.417 9")
                && l.contains("21436864.945 4")
                && l.contains("21436863.266 3")
                && l.contains("21436870.793 8")
        }),
        "Caso R07: la línea 123 debe mantener flags 4/3/8 en slots correctos"
    );
}

#[test]
fn test_pure_continuation_numeric_specific_cases() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let output = decompress_crinex_pure(&input).expect("Debe generar salida pura");
    let out_lines = normalize_lines(&output);

    assert!(
        out_lines.iter().any(|l| l.contains("2689.133 6") && l.contains("2095.433 9")),
        "Debe reconstruir 2095.433 en línea de continuación"
    );
    assert!(
        out_lines
            .iter()
            .any(|l| l.contains("1936.328 7") && l.contains("1508.94649")),
        "Debe reconstruir 1508.946 en línea de continuación"
    );
    assert!(
        out_lines
            .iter()
            .any(|l| l.contains("2156.035 5") && l.contains("1680.156 8")),
        "Debe reconstruir 1680.156 en línea de continuación"
    );
}

#[test]
fn test_pure_numeric_block_from_line_167_specific_cases() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let output = decompress_crinex_pure(&input).expect("Debe generar salida pura");
    let out_lines = normalize_lines(&output);

    assert!(
        out_lines
            .iter()
            .any(|l| l.contains("119008859.745") && l.contains("92734160.794")),
        "Debe reconstruir 119008859.745 y 92734160.794 en el bloque de línea 167"
    );
    assert!(
        out_lines.iter().any(|l| l.contains("22646615.813")),
        "Debe reconstruir 22646615.813 en el bloque de línea 167"
    );
    assert!(
        out_lines.iter().any(|l| l.contains("2689.031 6")),
        "Debe reconstruir 2689.031 en línea de continuación del bloque 167"
    );
    assert!(
        out_lines.iter().any(|l| l.contains("107788571.068")),
        "Debe reconstruir 107788571.068 en la línea siguiente del bloque 167"
    );
}

fn print_debug_for_line(
    target_line: usize,
    expected_line: &str,
    output_line: Option<&String>,
    rec: Option<&PureRustDebugRecord>,
) {
    eprintln!("========== DEBUG line {target_line} ==========");
    eprintln!("expected: {expected_line}");
    eprintln!(
        "output  : {}",
        output_line.map_or("<sin línea>".to_string(), |s| s.clone())
    );
    if let Some(r) = rec {
        eprintln!("epoch_index       : {}", r.epoch_index);
        eprintln!("diff_order        : {}", r.diff_order);
        eprintln!("satellite         : {}", r.satellite);
        eprintln!("compact_line      : {}", r.compact_line);
        eprintln!("value_tokens      : {:?}", r.value_tokens);
        eprintln!("value_columns     : {:?}", r.value_token_columns);
        eprintln!("raw_flags         : {:?}", r.raw_flags);
        eprintln!("raw_flag_columns  : {:?}", r.raw_flag_columns);
        eprintln!("raw_flag_tail_cols: {:?}", r.raw_flag_tail_columns);
        eprintln!("flag_tail         : {:?}", r.flag_tail);
        eprintln!("value_updates     : {:?}", r.value_updates);
        eprintln!("chosen_slots      : {:?}", r.chosen_slots);
        eprintln!("slot_flags(prev→next):");
        for sf in &r.slot_flags {
            eprintln!("  slot {}: {:?} -> {:?}", sf.slot, sf.prev, sf.next);
        }
        eprintln!(
            "rinex_line_1 (#{}) : {}",
            r.output_line_1, r.rinex_line_1
        );
        if let Some(line_2) = &r.rinex_line_2 {
            eprintln!(
                "rinex_line_2 (#{}) : {}",
                r.output_line_2.unwrap_or_default(),
                line_2
            );
        }
    } else {
        eprintln!("No se encontró registro debug para esa línea.");
    }
}

fn parse_obs_value_from_line(line: &str, field_index: usize) -> Option<f64> {
    let width = 16usize;
    let start = field_index * width;
    let end = ((field_index + 1) * width).min(line.len());
    if start >= line.len() || end <= start {
        return None;
    }
    let field = &line[start..end];
    let val = field.get(0..14).unwrap_or("").trim();
    if val.is_empty() {
        return None;
    }
    val.parse::<f64>().ok()
}

#[test]
#[ignore]
fn debug_line_115_flags() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let expected = std::fs::read_to_string("../samples/igm1252a.25o")
        .expect("Debe poder leerse el sample RINEX esperado");

    let (output, records) = decompress_crinex_pure_debug(&input).expect("Debe generar debug puro");
    let out_lines = normalize_lines(&output);
    let exp_lines = normalize_lines(&expected);

    let targets = [115usize, 116usize, 123usize];
    for line_no in targets {
        let expected_line = exp_lines
            .get(line_no - 1)
            .map_or("<línea esperada fuera de rango>", |s| s.as_str());
        let out_line = out_lines.get(line_no - 1);
        let rec = records.iter().find(|r| {
            r.output_line_1 == line_no || r.output_line_2.map(|n| n == line_no).unwrap_or(false)
        });
        print_debug_for_line(line_no, expected_line, out_line, rec);
    }

    // Este test es de diagnóstico; solo valida que haya material para inspección.
    assert!(
        records
            .iter()
            .any(|r| [115usize, 116usize, 123usize].contains(&r.output_line_1)
                || r.output_line_2
                    .map(|n| [115usize, 116usize, 123usize].contains(&n))
                    .unwrap_or(false)),
        "No se encontraron registros de debug para las líneas objetivo"
    );
}

#[test]
#[ignore]
fn debug_line_136_continuation_accumulation() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let expected = std::fs::read_to_string("../samples/igm1252a.25o")
        .expect("Debe poder leerse el sample RINEX esperado");

    let (output, records) = decompress_crinex_pure_debug(&input).expect("Debe generar debug puro");
    let out_lines = normalize_lines(&output);
    let exp_lines = normalize_lines(&expected);

    let targets = [136usize, 138usize, 140usize];
    for line_no in targets {
        let exp_line = exp_lines.get(line_no - 1).cloned().unwrap_or_default();
        let out_line = out_lines.get(line_no - 1).cloned().unwrap_or_default();
        let expected_value = parse_obs_value_from_line(&exp_line, 1);
        let rec = records.iter().find(|r| r.output_line_2 == Some(line_no));

        eprintln!("========== DEBUG acumulación line {line_no} ==========");
        eprintln!("expected: {exp_line}");
        eprintln!("output  : {out_line}");
        if let Some(r) = rec {
            eprintln!("satellite: {}", r.satellite);
            eprintln!("diff_order: {}", r.diff_order);
            if let Some(update) = r.value_updates.iter().find(|u| u.slot == 6) {
                eprintln!(
                    "slot={} prev={:?} delta={} result={:?} expected={:?}",
                    update.slot, update.previous, update.delta, update.result, expected_value
                );
            } else {
                eprintln!("No hubo update explícito para slot 6 en este registro");
            }
        } else {
            eprintln!("No se encontró registro debug con output_line_2={line_no}");
        }
    }
}

#[test]
#[ignore]
fn debug_line_167_continuation_accumulation() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let expected = std::fs::read_to_string("../samples/igm1252a.25o")
        .expect("Debe poder leerse el sample RINEX esperado");

    let (output, records) = decompress_crinex_pure_debug(&input).expect("Debe generar debug puro");
    let out_lines = normalize_lines(&output);
    let exp_lines = normalize_lines(&expected);

    let targets = [167usize, 168usize, 169usize];
    for line_no in targets {
        let exp_line = exp_lines.get(line_no - 1).cloned().unwrap_or_default();
        let out_line = out_lines.get(line_no - 1).cloned().unwrap_or_default();
        let expected_first = parse_obs_value_from_line(&exp_line, 0);
        let expected_second = parse_obs_value_from_line(&exp_line, 1);
        let expected_third = parse_obs_value_from_line(&exp_line, 2);
        let rec = records.iter().find(|r| {
            r.output_line_1 == line_no || r.output_line_2.map(|n| n == line_no).unwrap_or(false)
        });

        eprintln!("========== DEBUG acumulación line {line_no} ==========");
        eprintln!("expected: {exp_line}");
        eprintln!("output  : {out_line}");
        if let Some(r) = rec {
            eprintln!("satellite: {}", r.satellite);
            eprintln!("diff_order: {}", r.diff_order);
            eprintln!("compact_line: {}", r.compact_line);
            eprintln!("value_tokens: {:?}", r.value_tokens);
            eprintln!("chosen_slots: {:?}", r.chosen_slots);
            eprintln!(
                "expected fields[0..3]: {:?}, {:?}, {:?}",
                expected_first, expected_second, expected_third
            );
            for update in &r.value_updates {
                eprintln!(
                    "slot={} prev={:?} delta={} result={:?}",
                    update.slot, update.previous, update.delta, update.result
                );
            }
        } else {
            eprintln!("No se encontró registro debug para línea {line_no}");
        }
    }
}
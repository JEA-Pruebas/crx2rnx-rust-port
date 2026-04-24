use crx2rnx_port::{decompress_crinex_pure, inspect_crinex_pure};

#[test]
fn test_pure_header_removes_crinex_lines_and_keeps_rinex_header() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");

    let analysis = inspect_crinex_pure(&input).expect("Debe parsear header en modo puro");

    assert!(analysis.rinex_header.contains("RINEX VERSION / TYPE"));
    assert!(analysis.rinex_header.contains("END OF HEADER"));
    assert!(!analysis.rinex_header.contains("CRINEX VERS   / TYPE"));
    assert!(!analysis.rinex_header.contains("CRINEX PROG / DATE"));
}

#[test]
fn test_pure_detects_epochs_and_satellites() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");

    let analysis = inspect_crinex_pure(&input).expect("Debe inspeccionar epochs en modo puro");

    assert!(!analysis.epochs.is_empty());
    let first_epoch = &analysis.epochs[0];
    assert!(first_epoch.epoch_line.starts_with(" 25  9  9"));
    assert_eq!(first_epoch.satellites.len(), 15);
    assert!(first_epoch.satellites.contains(&"G01".to_string()));
    assert!(first_epoch.satellites.contains(&"R24".to_string()));
}

#[test]
fn test_pure_multi_epoch_multi_satellite_delta_accumulation_prefixes() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let expected = std::fs::read_to_string("../samples/igm1252a.25o")
        .expect("Debe poder leerse el sample RINEX esperado");

    let output = decompress_crinex_pure(&input).expect("Modo puro parcial debe generar salida");

    let out_norm = output.replace("\r\n", "\n");
    let exp_norm = expected.replace("\r\n", "\n");

    // Epochs 0,1,2 presentes (marcador inter-epoch procesado).
    assert!(out_norm.contains(" 25  9  9  0  0  0.0000000"));
    assert!(out_norm.contains(" 25  9  9  0  0  1.0000000"));
    assert!(out_norm.contains(" 25  9  9  0  0  2.0000000"));

    // G01 repetido en múltiples epochs.
    assert!(out_norm.contains("119019616.473"));
    assert!(out_norm.contains("119016927.190"));
    // G02 repetido en múltiples epochs (al menos los dos primeros).
    assert!(out_norm.contains("107796317.646"));
    assert!(out_norm.contains("107794380.462"));

    // Observable reconstruida por acumulación/delta (ejemplo D2 de G01 epoch 1).
    assert!(out_norm.contains("2095.536"));

    // Todas estas referencias existen en salida esperada.
    assert!(exp_norm.contains(" 25  9  9  0  0  2.0000000"));
    assert!(exp_norm.contains("2095.536"));
}

#[test]
fn test_pure_first_observation_lines_include_initial_flags() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let output = decompress_crinex_pure(&input).expect("Modo puro parcial debe generar salida");
    let out_norm = output.replace("\r\n", "\n");

    assert!(out_norm.contains("119019616.473 6"));
    assert!(out_norm.contains("92742542.658 9"));
    assert!(out_norm.contains("22648663.242 6"));
}

#[test]
fn test_pure_preserves_empty_slots_and_continuation_layout_for_3207_case() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let output = decompress_crinex_pure(&input).expect("Modo puro parcial debe generar salida");
    let out_norm = output.replace("\r\n", "\n");
    let lines = out_norm.lines().collect::<Vec<_>>();

    let idx_main = lines
        .iter()
        .position(|l| l.contains("115211522.654") && l.contains("21552672.820"))
        .expect("Debe existir línea principal del caso 3207.539");
    let idx_cont = idx_main + 1;

    let main = lines[idx_main];
    let cont = lines
        .get(idx_cont)
        .copied()
        .expect("Debe existir línea de continuación");

    // Debe quedar en continuación, no adelantado a la línea principal.
    assert!(!main.contains("3207.539"));
    assert!(cont.contains("3207.539 4"));
    assert!(cont.contains("38.900"));
}

#[test]
fn test_pure_keeps_21435534_on_main_line_and_minus3576_on_continuation() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let output = decompress_crinex_pure(&input).expect("Modo puro parcial debe generar salida");
    let out_norm = output.replace("\r\n", "\n");
    let lines = out_norm.lines().collect::<Vec<_>>();

    let idx_main = lines
        .iter()
        .position(|l| l.contains("114746237.112") && l.contains("21435526.336"))
        .expect("Debe existir línea principal del caso 21435534.504");
    let idx_cont = idx_main + 1;

    let main = lines[idx_main];
    let cont = lines
        .get(idx_cont)
        .copied()
        .expect("Debe existir línea de continuación");

    assert!(main.contains("21435534.504 8"));
    assert!(!cont.contains("21435534.504 8"));
    assert!(cont.contains("-3576.688 3"));
}

#[test]
fn test_pure_epoch_line_has_satellite_count_attached_to_first_satellite() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let output = decompress_crinex_pure(&input).expect("Modo puro parcial debe generar salida");
    let out_norm = output.replace("\r\n", "\n");

    assert!(out_norm.contains(" 25  9  9  0  0  1.0000000  0 15G01"));
    assert!(!out_norm.contains(" 25  9  9  0  0  1.0000000  0 15 G01"));
}

#[test]
fn test_pure_epoch1_line71_flags_for_first_satellite() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let output = decompress_crinex_pure(&input).expect("Modo puro parcial debe generar salida");
    let out_norm = output.replace("\r\n", "\n");

    let line = out_norm
        .lines()
        .find(|l| l.contains("119016927.190") && l.contains("22648151.602"))
        .expect("Debe existir la línea de observaciones del primer satélite en epoch 1");

    assert!(line.contains("119016927.190 6"));
    assert!(line.contains("92740447.122 9"));
    assert!(line.contains("22648151.602 6"));
}

#[test]
fn test_pure_later_epoch_flag_regressions_line83_84_and_91() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let output = decompress_crinex_pure(&input).expect("Modo puro parcial debe generar salida");
    let out_norm = output.replace("\r\n", "\n");
    let lines = out_norm.lines().collect::<Vec<_>>();

    let idx83 = lines
        .iter()
        .position(|l| l.contains("126244825.732") && l.contains("24023611.234"))
        .expect("Debe existir la línea equivalente al caso 83");
    let line83 = lines[idx83];
    let line84 = lines
        .get(idx83 + 1)
        .copied()
        .expect("Debe existir continuación");

    assert!(line83.contains("126244825.732 5"));
    assert!(line83.contains("24023618.72346"));
    assert!(line84.contains("1843.976 9"));

    let line91_like = lines
        .iter()
        .find(|l| l.contains("114749814.022"))
        .expect("Debe existir línea equivalente al caso 91");
    assert!(line91_like.contains("114749814.022 3"));
}

#[test]
fn test_pure_first_numeric_divergence_case_reconstructs_line103_value() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let output = decompress_crinex_pure(&input).expect("Modo puro parcial debe generar salida");
    let out_norm = output.replace("\r\n", "\n");

    assert!(out_norm.contains("119014237.993"));
    assert!(!out_norm.contains("119016927.276"));
}
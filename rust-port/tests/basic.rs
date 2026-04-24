use crx2rnx_port::decompress_crinex;

fn normalize_rinex_for_assertion(s: &str) -> String {
    let normalized = s.replace("\r\n", "\n");
    let normalized_lines = normalized
        .lines()
        .map(|line| line.trim_end_matches(' ').to_string())
        .collect::<Vec<String>>();

    let mut out = normalized_lines.join("\n");
    out.push('\n');
    out
}

#[test]
fn test_basic_crinex_sample_matches_expected_output() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d")
        .expect("Debe poder leerse el sample CRINEX de entrada");
    let expected = std::fs::read_to_string("../samples/igm1252a.25o")
        .expect("Debe poder leerse el sample RINEX esperado");

        let output =
            decompress_crinex(&input).expect("La descompresión del sample real debe funcionar");

        assert_eq!(
            normalize_rinex_for_assertion(&output),
            normalize_rinex_for_assertion(&expected)
    );
}

#[test]
fn test_rejects_non_compact_rinex_input() {
    let err = decompress_crinex("not a compact rinex file")
        .expect_err("Debe devolver error para entradas inválidas");
    assert!(err.message.contains("END OF HEADER") || err.message.contains("Compact RINEX"));
}
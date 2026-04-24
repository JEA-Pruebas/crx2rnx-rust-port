use crx2rnx_port::decompress_crinex;

#[test]
fn test_basic_crinex() {
    let input = std::fs::read_to_string("../samples/igm1252a.25d").unwrap();
    let expected = std::fs::read_to_string("../samples/igm1252a.25o").unwrap();

    let output = decompress_crinex(&input).unwrap();

    assert_eq!(output, expected);
}
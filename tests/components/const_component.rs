use nanotekspice::Circuit;
use test_generator::test_resources;

#[test_resources("tests/.nts/true.nts")]
fn always_returns_true(path: &str) {
    let mut circuit: Circuit = std::fs::read_to_string(path).unwrap().parse().unwrap();

    assert_eq!(circuit.get_output("out").unwrap(), "1");

    for _ in 0..3 {
        circuit.simulate();

        assert_eq!(circuit.get_output("out").unwrap(), "1");
    }
}

#[test_resources("tests/.nts/false.nts")]
fn always_returns_false(path: &str) {
    let mut circuit: Circuit = std::fs::read_to_string(path).unwrap().parse().unwrap();

    assert_eq!(circuit.get_output("out").unwrap(), "0");

    for _ in 0..3 {
        circuit.simulate();

        assert_eq!(circuit.get_output("out").unwrap(), "0");
    }
}

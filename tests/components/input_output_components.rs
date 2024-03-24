use nanotekspice::Circuit;
use test_generator::test_resources;

#[test_resources("tests/.nts/input_output.nts")]
fn get_an_input_from_prompt(path: &str) {
    let mut circuit: Circuit = std::fs::read_to_string(path).unwrap().parse().unwrap();

    assert_eq!(circuit.get_input("in").unwrap(), "U");
    assert_eq!(circuit.get_output("out").unwrap(), "U");

    for state in ["0", "1", "U"] {
        circuit.set_value("in", state).unwrap();

        for _ in 0..3 {
            circuit.simulate();

            assert_eq!(circuit.get_input("in").unwrap(), state);
            assert_eq!(circuit.get_output("out").unwrap(), state);
        }
    }
}

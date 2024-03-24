use nanotekspice::Circuit;
use test_generator::test_resources;

#[test_resources("tests/.nts/clock.nts")]
fn works_same_as_input_for_the_next_tick(path: &str) {
    let mut circuit: Circuit = std::fs::read_to_string(path).unwrap().parse().unwrap();

    assert_eq!(circuit.get_input("cl").unwrap(), "U");
    assert_eq!(circuit.get_output("out").unwrap(), "U");

    for state in ["0", "1", "U"] {
        for _ in 0..3 {
            circuit.set_value("cl", state).unwrap();
            circuit.simulate();

            assert_eq!(circuit.get_input("cl").unwrap(), state);
            assert_eq!(circuit.get_output("out").unwrap(), state);
        }
    }
}

#[test_resources("tests/.nts/clock.nts")]
fn invert_state_at_each_simulate(path: &str) {
    let mut circuit: Circuit = std::fs::read_to_string(path).unwrap().parse().unwrap();

    circuit.set_value("cl", "0").unwrap();

    for state in ["0", "1", "0", "1", "0", "1"] {
        circuit.simulate();
        assert_eq!(circuit.get_input("cl").unwrap(), state);
        assert_eq!(circuit.get_output("out").unwrap(), state);
    }
}
#[test_resources("tests/.nts/clock.nts")]
fn does_not_invert_undefined_state(path: &str) {
    let mut circuit: Circuit = std::fs::read_to_string(path).unwrap().parse().unwrap();

    for _ in 0..5 {
        circuit.simulate();
        assert_eq!(circuit.get_input("cl").unwrap(), "U");
        assert_eq!(circuit.get_output("out").unwrap(), "U");
    }
}

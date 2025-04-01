use sec::sec_state_machine::State;
use sec::sec_state_machine::ingestion::retrieval::Retrieval;

#[test]
fn should_return_true_when_retrieval_state_has_computed_the_output() {
    let mut retrieval_state = Retrieval::default();

    let expected_result = true;

    retrieval_state.compute_output_data();
    let result = retrieval_state.has_output_data_been_computed();

    assert_eq!(result, expected_result);
}

#[test]
fn should_return_true_when_reference_of_retrieval_state_has_computed_the_output() {
    let ref_to_retrieval_state = &mut Retrieval::default();

    let expected_result = true;

    ref_to_retrieval_state.compute_output_data();
    let result = ref_to_retrieval_state.has_output_data_been_computed();

    assert_eq!(result, expected_result);
}

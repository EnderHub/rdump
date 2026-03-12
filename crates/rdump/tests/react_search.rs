mod common;
use common::assert_public_support_matrix;
use rdump::support_matrix::react_shared_cases;

#[test]
fn test_react_shared_behavior_matrix() {
    assert_public_support_matrix(react_shared_cases());
}

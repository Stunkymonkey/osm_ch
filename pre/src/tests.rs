use super::*;

#[test]
fn sort_source_test() {
    let mut source: Vec<u32> = vec![0, 2, 3, 4, 0, 4, 3, 4];
    let mut target: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let mut weight: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let mut kind: Vec<u8> = vec![0, 1, 2, 0, 1, 2, 0, 1];

    sort_source(&mut source, &mut target, &mut weight, &mut kind);

    assert_eq!(source[0], 0);
    assert_eq!(source[1], 0);
    assert_eq!(source[2], 2);
    assert_eq!(source[3], 3);
    assert_eq!(source[4], 3);
    assert_eq!(source[5], 4);
    assert_eq!(source[6], 4);
    assert_eq!(source[7], 4);

    assert_eq!(target[0], 0);
    assert_eq!(target[1], 4);
    assert_eq!(target[2], 1);
    assert_eq!(target[3], 2);
    assert_eq!(target[4], 6);
    assert_eq!(target[5], 3);
    assert_eq!(target[6], 5);
    assert_eq!(target[7], 7);

    assert_eq!(weight[0], 0);
    assert_eq!(weight[1], 4);
    assert_eq!(weight[2], 1);
    assert_eq!(weight[3], 2);
    assert_eq!(weight[4], 6);
    assert_eq!(weight[5], 3);
    assert_eq!(weight[6], 5);
    assert_eq!(weight[7], 7);

    assert_eq!(kind[0], 0);
    assert_eq!(kind[1], 1);
    assert_eq!(kind[2], 1);
    assert_eq!(kind[3], 2);
    assert_eq!(kind[4], 0);
    assert_eq!(kind[5], 0);
    assert_eq!(kind[6], 2);
    assert_eq!(kind[7], 1);
}

#[test]
fn fill_offset_test() {
    let mut offset_test = vec![0; 7];
    let sources: Vec<u32> = vec![0, 0, 0, 2, 3, 4, 4, 4, 6];

    fill_offset(&sources, &mut offset_test);

    //1 is not a valid node
    assert_eq!(offset_test[0], 0);
    assert_eq!(offset_test[1], 3);
    assert_eq!(offset_test[2], 3);
    assert_eq!(offset_test[3], 4);
    assert_eq!(offset_test[4], 5);
    assert_eq!(offset_test[5], 8);
    assert_eq!(offset_test[6], 8);
}

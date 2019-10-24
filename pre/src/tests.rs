use super::*;

#[test]
fn fill_offset_test() {
    let mut offset_test = vec![0; 8];
    let mut ways = Vec::<Way>::new();
    ways.push(Way {
        source: 0,
        target: 1,
        weight: 2,
        kind: 1,
    });
    ways.push(Way {
        source: 0,
        target: 1,
        weight: 2,
        kind: 1,
    });
    ways.push(Way {
        source: 0,
        target: 1,
        weight: 2,
        kind: 1,
    });
    ways.push(Way {
        source: 2,
        target: 1,
        weight: 2,
        kind: 1,
    });
    ways.push(Way {
        source: 3,
        target: 1,
        weight: 2,
        kind: 1,
    });
    ways.push(Way {
        source: 4,
        target: 1,
        weight: 2,
        kind: 1,
    });
    ways.push(Way {
        source: 4,
        target: 1,
        weight: 2,
        kind: 1,
    });
    ways.push(Way {
        source: 4,
        target: 1,
        weight: 2,
        kind: 1,
    });
    ways.push(Way {
        source: 6,
        target: 1,
        weight: 2,
        kind: 1,
    });
    fill_offset(&ways, &mut offset_test);

    println!("{:?}", offset_test);

    assert_eq!(offset_test[0], 0);
    assert_eq!(offset_test[1], 3);
    assert_eq!(offset_test[2], 3);
    assert_eq!(offset_test[3], 4);
    assert_eq!(offset_test[4], 5);
    assert_eq!(offset_test[5], 8);
    assert_eq!(offset_test[6], 8);
    assert_eq!(offset_test[7], 9);
}

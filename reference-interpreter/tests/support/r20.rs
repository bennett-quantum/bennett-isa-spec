//! Reviewer bytes and independently reconstructed §9.2 pins (r20_pins.py).
pub const CASES: [(&str, &str, &str); 2] = [
    (
        include_str!("r20-const-width-18446744073709551614.bisa"),
        "9146b47a91b5cade7648cdb4386bab0fc399048bd754674cb4745e79eeccb318",
        "254bf985f88a26f94c13ff4e503cb1fff95ea5d4452f17422e194e73bb6eb367",
    ),
    (
        include_str!("r20-const-width-18446744073709551615.bisa"),
        "97eb756124446d26628e2abbe5fb74223cbad5eb7902aacd31b7a309069f48f1",
        "94416b11560b138ee17b68f5aa930d73c5c9521e343a533842a7d0d6aa0a8344",
    ),
];

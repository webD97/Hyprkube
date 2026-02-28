const CHARSET: &[u8] = b"abcdefghklmnpqrstuvwxyz123456789";

pub(crate) fn random_id(len: usize) -> String {
    use rand::RngExt as _;

    let mut rng = rand::rng();

    (0..len)
        .map(|_| {
            let i = rng.random_range(0..CHARSET.len());
            CHARSET[i] as char
        })
        .collect()
}

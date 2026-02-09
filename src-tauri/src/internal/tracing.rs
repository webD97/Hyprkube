use rand::{rng, RngExt as _};

pub fn set_span_request_id() {
    tracing::Span::current().record("request_id", generate_base36_id(6));
}

fn generate_base36_id(len: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rng();

    (0..len)
        .map(|_| {
            let i = rng.random_range(0..36);
            CHARSET[i] as char
        })
        .collect()
}

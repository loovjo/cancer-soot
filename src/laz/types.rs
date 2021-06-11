#[derive(Clone, Debug)]
pub enum LazValue {
    Byte(u8),
    Char(char),
    Unsigned(u64),
    Signed(i64),

    Array(Vec<LazValue>),
    String(String),
}


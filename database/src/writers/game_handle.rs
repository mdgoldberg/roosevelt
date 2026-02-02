#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameHandle(pub(crate) i64);

impl GameHandle {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }
}

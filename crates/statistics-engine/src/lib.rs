#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AnswerTotals {
    pub explored: u32,
    pub correct: u32,
    pub mistakes: u32,
}

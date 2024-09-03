#[derive(Debug, Clone, Copy)]
pub struct Span(pub usize, pub usize);
impl Span {
    pub fn concat(self, other: Self) -> Self {
        Span(usize::min(self.0, other.0), usize::max(self.1, other.1))
    }
    pub const fn range(self) -> std::ops::Range<usize> {
        self.0..self.1
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpanOf<T> {
    pub value: T,
    pub span: Span,
}
impl<T> SpanOf<T> {
    pub const fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
    pub fn combine<T1, O>(self, other: SpanOf<T1>, func: impl FnOnce(T, T1) -> O) -> SpanOf<O> {
        SpanOf::new(func(self.value, other.value), self.span.concat(other.span))
    }
}

use std::{rc::Rc, str::CharIndices};

use error::Error;
use span::*;

mod error;
mod literal;
mod span;

#[derive(Clone)]
pub struct Source {
    source: Rc<String>,
    iter: CharIndices<'static>,
}
impl Source {
    pub fn new(source: String) -> Self {
        // SAFETY: Do not move iter out of Source struct
        // Do not mutate source field, instead create a new Source struct with the new source string
        let source = Rc::new(source);
        let iter: CharIndices<'static> = unsafe { std::mem::transmute(source.char_indices()) };
        Self { source, iter }
    }
    pub fn source(&self) -> &Rc<String> {
        &self.source
    }
    pub fn iter(&self) -> &CharIndices<'static> {
        &self.iter
    }
    pub fn next() -> impl Parser<SpanOf<char>> {
        |mut src: Source| {
            Ok(src
                .iter
                .next()
                .map(|(i, ch)| (SpanOf::new(ch, Span(i, i + ch.len_utf8())), src)))
        }
    }
    pub fn next_if(condition: impl FnOnce(char) -> bool) -> impl Parser<SpanOf<char>> {
        Self::next().then(|ch| {
            move |src: Source| {
                if condition(ch.value) {
                    return Ok(Some((ch, src)));
                }
                Ok(None)
            }
        })
    }
}

pub type Result<T> = std::result::Result<Option<(T, Source)>, Error>;

pub trait Parser<O> {
    fn parse(self, source: Source) -> Result<O>;
    fn map<O1>(self, func: impl FnOnce(O) -> O1) -> impl Parser<O1>;
    fn then<O1, P1: Parser<O1>>(self, func: impl FnOnce(O) -> P1) -> impl Parser<O1>;
    fn or<P1: Parser<O>>(self, func: impl FnOnce() -> P1) -> impl Parser<O>;
}
impl<F: FnOnce(Source) -> Result<O>, O> Parser<O> for F {
    fn parse(self, source: Source) -> Result<O> {
        self(source)
    }
    fn map<O1>(self, func: impl FnOnce(O) -> O1) -> impl Parser<O1> {
        move |src: Source| self.parse(src).map(|a| a.map(|(a, src)| (func(a), src)))
    }
    fn then<O1, P1: Parser<O1>>(self, func: impl FnOnce(O) -> P1) -> impl Parser<O1> {
        move |src: Source| match self.parse(src)? {
            Some((a, src)) => func(a).parse(src),
            None => Ok(None),
        }
    }
    fn or<P1: Parser<O>>(self, func: impl FnOnce() -> P1) -> impl Parser<O> {
        move |src: Source| match self.parse(src.clone()) {
            Ok(None) => func().parse(src),
            a => a,
        }
    }
}

pub fn fold<V, P: Parser<O>, O>(
    mut parser: impl FnMut() -> P,
    mut value: V,
    mut accumulator: impl FnMut(V, O) -> V,
) -> impl Parser<V> {
    move |mut src: Source| {
        while let Some((a, next)) = parser().parse(src.clone())? {
            src = next;
            value = accumulator(value, a);
        }
        Ok(Some((value, src)))
    }
}
pub fn fold_until<V, P: Parser<O>, O>(
    mut parser: impl FnMut() -> P,
    mut value: V,
    mut accumulator: impl FnMut(V, O) -> (V, bool),
) -> impl Parser<V> {
    move |mut src: Source| {
        while let Some((a, next)) = parser().parse(src.clone())? {
            src = next;
            let (next, is_break) = accumulator(value, a);
            value = next;
            if is_break {
                break;
            }
        }
        Ok(Some((value, src)))
    }
}
pub fn reduce<P: Parser<O>, O>(
    mut parser: impl FnMut() -> P,
    mut accumulator: impl FnMut(O, O) -> O,
) -> impl Parser<O> {
    move |src: Source| {
        let Some((mut value, mut src)) = parser().parse(src)? else {
            return Ok(None);
        };
        while let Some((a, next)) = parser().parse(src.clone())? {
            src = next;
            value = accumulator(value, a);
        }
        Ok(Some((value, src)))
    }
}

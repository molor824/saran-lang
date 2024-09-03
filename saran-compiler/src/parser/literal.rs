use super::*;

fn match_str(value: &'static str) -> impl Parser<Span> {
    move |mut src: Source| {
        let mut span = Option::<Span>::None;
        for ch in value.chars() {
            match Source::next_if(|ch1| ch == ch1).parse(src)? {
                Some((ch, next)) => {
                    src = next;
                    span = Some(span.map_or(ch.span, |s| s.concat(ch.span)));
                }
                None => return Ok(None),
            }
        }
        Ok(span.map(|s| (s, src)))
    }
}

fn line_comment() -> impl Parser<()> {
    match_str("--").then(|_| {
        fold_until(
            || {
                match_str("\n")
                    .map(|_| true)
                    .or(|| Source::next().map(|_| false))
            },
            (),
            |_, is_break| ((), is_break),
        )
    })
}

fn block_comment() -> impl Parser<()> {
    match_str("--[[").then(|_| {
        fold_until(
            || {
                match_str("]]")
                    .map(|_| true)
                    .or(|| Source::next().map(|_| false))
            },
            (),
            |_, is_break| ((), is_break),
        )
    })
}

fn whitespaces() -> impl Parser<()> {
    Source::next_if(|ch| ch.is_whitespace()).map(|_| ())
}

fn skip() -> impl Parser<()> {
    reduce(
        || whitespaces().or(|| block_comment()).or(|| line_comment()),
        |_, _| (),
    )
}

fn first_ident_ch() -> impl Parser<Span> {
    Source::next_if(|ch| ch.is_alphabetic() || ch == '_').map(|ch| ch.span)
}
fn ident_ch() -> impl Parser<Span> {
    Source::next_if(|ch| ch.is_alphanumeric() || ch == '_').map(|ch| ch.span)
}
fn ident_prime_ch() -> impl Parser<Span> {
    match_str("'")
}
fn ident() -> impl Parser<SpanOf<String>> {
    skip()
        .then(|_| first_ident_ch())
        .then(|span| fold(ident_ch, span, Span::concat))
        .then(|span| fold(ident_prime_ch, span, Span::concat))
        .then(|span| {
            move |src: Source| {
                Ok(Some((
                    SpanOf::new(src.source()[span.range()].to_string(), span),
                    src,
                )))
            }
        })
}

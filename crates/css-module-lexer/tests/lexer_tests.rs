use super::*;
fn token_text(input: &str, token: Token) -> &str {
  Lexer::slice_range(input, &token.range).expect("test setup must produce the expected value")
}

#[test]
fn streaming_tokenizer_consumes_one_to_six_hex_escape_digits() {
  for input in [
    r"\1 ",
    r"\12 ",
    r"\123 ",
    r"\1234 ",
    r"\12345 ",
    r"\123456 ",
  ] {
    let mut lexer = Lexer::new(input, ());
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident);
    assert_eq!(token_text(input, token), input);
    assert_eq!(lexer.next_token().kind, TokenKind::Eof);
  }
}

#[test]
fn streaming_tokenizer_keeps_consecutive_comments_distinct() {
  let input = "/** first **//** second **/ \tfoo";
  let mut lexer = Lexer::new(input, ());
  let first = lexer.next_token();
  let second = lexer.next_token();
  let whitespace = lexer.next_token();
  let ident = lexer.next_token();
  assert_eq!(first.kind, TokenKind::Comment);
  assert_eq!(second.kind, TokenKind::Comment);
  assert_eq!(token_text(input, first), "/** first **/");
  assert_eq!(token_text(input, second), "/** second **/");
  assert_eq!(whitespace.kind, TokenKind::WhiteSpace);
  assert_eq!(ident.kind, TokenKind::Ident);

  let mut stream_lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut stream_lexer);
  let significant = stream.next_parser_token();
  assert_eq!(significant.token.kind, TokenKind::Ident);
  assert_eq!(significant.leading.first_comment_start, Some(0));
  assert!(significant.leading.has_whitespace());
  assert_eq!(significant.leading.range, Range::new(0, 29));
}

#[test]
fn streaming_tokenizer_reports_eof_tokens_after_unterminated_inputs() {
  let mut ident = Lexer::new("ident", ());
  assert_eq!(ident.next_token().kind, TokenKind::Ident);
  assert_eq!(ident.next_token().kind, TokenKind::Eof);

  let mut string = Lexer::new("'unterminated", ());
  assert_eq!(string.next_token().kind, TokenKind::BadString);
  assert_eq!(string.next_token().kind, TokenKind::Eof);

  let mut comment = Lexer::new("/* unterminated", ());
  assert_eq!(comment.next_token().kind, TokenKind::BadComment);
  assert_eq!(comment.next_token().kind, TokenKind::Eof);
}

#[test]
fn streaming_tokenizer_does_not_treat_string_comments_as_trivia() {
  let input = r#""/* not a comment */""#;
  let mut lexer = Lexer::new(input, ());
  let token = lexer.next_token();
  assert_eq!(token.kind, TokenKind::QuotedString);
  assert_eq!(token_text(input, token), input);
  assert_eq!(lexer.next_token().kind, TokenKind::Eof);
}

#[test]
fn streaming_tokenizer_scans_escaped_url_content_as_one_token() {
  let input = r"url(https:\2f\2fexample.com\2fimage.png)";
  let mut lexer = Lexer::new(input, ());
  let token = lexer.next_token();
  assert_eq!(token.kind, TokenKind::Url);
  assert_eq!(token_text(input, token), input);
  assert_eq!(
    Lexer::slice_range(input, &token.value_range)
      .expect("test setup must produce the expected value"),
    r"https:\2f\2fexample.com\2fimage.png"
  );
  assert_eq!(lexer.next_token().kind, TokenKind::Eof);
}

#[test]
fn streaming_tokenizer_marks_token_flags() {
  let mut plain = Lexer::new("plain", ());
  let plain = plain.next_token();
  assert!(plain.flags.is_ascii());
  assert!(!plain.flags.has_escape());
  assert!(!plain.flags.has_null());

  let mut escaped = Lexer::new(r"a\2db", ());
  let escaped = escaped.next_token();
  assert!(escaped.flags.is_ascii());
  assert!(escaped.flags.has_escape());
  assert!(!escaped.flags.has_null());

  let mut nul = Lexer::new("a\0b", ());
  let nul = nul.next_token();
  assert!(nul.flags.is_ascii());
  assert!(!nul.flags.has_escape());
  assert!(nul.flags.has_null());

  let mut unicode = Lexer::new("café", ());
  let unicode = unicode.next_token();
  assert!(!unicode.flags.is_ascii());
  assert!(!unicode.flags.has_escape());
  assert!(!unicode.flags.has_null());

  let mut unicode_url = Lexer::new("url(café)", ());
  let unicode_url = unicode_url.next_token();
  assert_eq!(unicode_url.kind, TokenKind::Url);
  assert!(!unicode_url.flags.is_ascii());

  let mut unterminated_comment = Lexer::new("/*\0", ());
  let unterminated_comment = unterminated_comment.next_token();
  assert_eq!(unterminated_comment.kind, TokenKind::BadComment);
  assert!(unterminated_comment.flags.has_null());
}

#[test]
fn streaming_tokenizer_handles_escaped_url_names_and_string_continuations() {
  let input = r"u\72l(foo)";
  let mut lexer = Lexer::new(input, ());
  assert_eq!(lexer.next_token().kind, TokenKind::Url);
  assert_eq!(lexer.next_token().kind, TokenKind::Eof);

  let input: String = ['"', 'a', '\\', '\n', 'b', '"'].into_iter().collect();
  let mut lexer = Lexer::new(&input, ());
  let token = lexer.next_token();
  assert_eq!(token.kind, TokenKind::QuotedString);
  assert_eq!(token_text(&input, token), input);
}

#[test]
fn token_stream_preserves_nested_value_at_rule_punctuation() {
  let input = "@value (foo:bar, func(a,b; c)) from \"theme.css\";";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  let mut kinds = Vec::new();
  loop {
    let token = stream.next_parser_token().token;
    kinds.push(token.kind);
    if token.kind == TokenKind::Eof {
      break;
    }
  }
  assert_eq!(
    kinds,
    vec![
      TokenKind::AtKeyword,
      TokenKind::LeftParenthesis,
      TokenKind::Ident,
      TokenKind::Colon,
      TokenKind::Ident,
      TokenKind::Comma,
      TokenKind::Function,
      TokenKind::Ident,
      TokenKind::Comma,
      TokenKind::Ident,
      TokenKind::Semicolon,
      TokenKind::Ident,
      TokenKind::RightParenthesis,
      TokenKind::RightParenthesis,
      TokenKind::Ident,
      TokenKind::QuotedString,
      TokenKind::Semicolon,
      TokenKind::Eof,
    ]
  );
}

#[test]
fn token_stream_folds_trivia_and_preserves_pure_comments() {
  let input = " \t/** first **/foo";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  let item = stream.next(false);
  assert_eq!(item.token.kind, TokenKind::Ident);
  assert!(item.leading.has_whitespace());
  assert_eq!(item.leading.first_comment_start, Some(2));

  let input = "/** magic **/ foo";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  assert_eq!(stream.next(true).token.kind, TokenKind::Comment);
  let item = stream.next(true);
  assert_eq!(item.token.kind, TokenKind::Ident);
  assert!(item.leading.has_whitespace());
}

#[test]
fn token_stream_peeks_past_comments_without_rescanning() {
  let mut lexer = Lexer::new("local/**/.foo", ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  assert_eq!(stream.next(true).token.kind, TokenKind::Ident);
  let next = stream.peek_significant_skipping_comments(true);
  assert_eq!(next.token.kind, TokenKind::Delim);
  let scan_after_peek = stream.lexer().scan_pos();
  assert_eq!(stream.next(true).token.kind, TokenKind::Comment);
  assert_eq!(stream.next(true).token.kind, TokenKind::Delim);
  assert_eq!(stream.lexer().scan_pos(), scan_after_peek);
}

#[test]
fn slice_range_rejects_invalid_utf8_boundaries() {
  let input = "aéz";
  assert_eq!(Lexer::slice_range(input, &Range::new(1, 3)), Some("é"));
  assert!(Lexer::slice_range(input, &Range::new(1, 2)).is_none());
  assert!(Lexer::slice_range(input, &Range::new(2, 3)).is_none());
  assert!(Lexer::slice_range(input, &Range::new(0, 5)).is_none());
  assert!(Lexer::slice_range(input, &Range::new(3, 1)).is_none());
}

#[test]
fn token_stream_cursors_are_monotonic_and_cover_the_input() {
  for input in [
    ".a { color: red; }",
    ".a { width: 10px; }",
    ".a { color: red; background: blue; }",
    ":local(.a) {}",
    ":local/**/.a {}",
    ":global/**/#id {}",
    ".a { color: var(--color); }",
    ".a { background: url(\"./a.png\"); }",
    ".a { color: rgb(1, 2, 3); }",
  ] {
    let mut lexer = Lexer::new(input, ());
    let mut stream = TokenStream::from_lexer(&mut lexer);
    let mut previous_scan = 0;
    let mut previous_consumed = 0;
    let mut previous_end = 0;
    loop {
      let item = stream.next_parser_token();
      let scan = stream.lexer().scan_pos();
      let consumed = stream.consumed_pos();
      assert!(scan >= previous_scan, "scan_pos went backward in {input:?}");
      assert!(
        consumed >= previous_consumed,
        "consumed_pos went backward in {input:?}"
      );
      assert!(
        consumed <= scan,
        "consumed_pos ({consumed}) exceeded scan_pos ({scan}) in {input:?}"
      );
      assert!(
        item.token.range.start >= previous_end,
        "token ranges are not increasing in {input:?}"
      );
      previous_end = item.token.range.end;
      previous_scan = scan;
      previous_consumed = consumed;
      if item.token.kind == TokenKind::Eof {
        break;
      }
    }
    assert_eq!(
      previous_consumed,
      input.len() as Pos,
      "did not consume the whole input: {input:?}"
    );
    assert_eq!(
      previous_scan,
      input.len() as Pos,
      "did not scan the whole input: {input:?}"
    );
  }
}

#[test]
fn token_stream_single_peek_does_not_rescan() {
  let mut lexer = Lexer::new("color:red;", ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  assert_eq!(stream.next(true).token.kind, TokenKind::Ident);
  assert_eq!(stream.next(true).token.kind, TokenKind::Colon);
  let red = stream.peek_significant_skipping_comments(true).token;
  assert_eq!(red.kind, TokenKind::Ident);
  let scan_before = stream.lexer().scan_pos();
  assert_eq!(stream.next(true).token.kind, TokenKind::Ident);
  assert_eq!(
    stream.lexer().scan_pos(),
    scan_before,
    "consuming the buffered token must not advance the scanner"
  );
  assert_eq!(stream.next(true).token.kind, TokenKind::Semicolon);
}

#[test]
fn token_stream_comment_separated_mode_stays_buffered_in_source_order() {
  // `:local/**/.foo` keeps the comment and `.foo` in the buffer so the
  // dependency scanner consumes them exactly once in source order.
  let mut lexer = Lexer::new(":local/**/.foo", ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  assert_eq!(stream.next(true).token.kind, TokenKind::Colon);
  assert_eq!(stream.next(true).token.kind, TokenKind::Ident);
  let delim = stream.peek_significant_skipping_comments(true).token;
  assert_eq!(delim.kind, TokenKind::Delim);
  let scan_after_peek = stream.lexer().scan_pos();
  assert_eq!(stream.next(true).token.kind, TokenKind::Comment);
  assert_eq!(stream.next(true).token.kind, TokenKind::Delim);
  assert_eq!(
    stream.lexer().scan_pos(),
    scan_after_peek,
    "the buffered comment and delim must be consumed without rescanning"
  );
  assert_eq!(stream.next(true).token.kind, TokenKind::Ident);
  assert_eq!(stream.next(true).token.kind, TokenKind::Eof);
}

#[test]
fn generic_value_fast_forward_keeps_raw_depth_across_candidates() {
  let input = "calc(1px + imported) url(a.png);";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);

  stream.fast_forward_generic_value_if_buffer_empty(
    false,
    false,
    false,
    |ident| ident == "imported",
    |_| false,
  );
  assert_eq!(stream.consumed_pos(), 11);
  assert_eq!(stream.next(false).token.kind, TokenKind::Ident);

  stream.fast_forward_generic_value_if_buffer_empty(false, false, false, |_| false, |_| false);
  assert_eq!(stream.consumed_pos(), 21);
  assert_eq!(stream.next(false).token.kind, TokenKind::Url);

  stream.fast_forward_generic_value_if_buffer_empty(false, false, false, |_| false, |_| false);
  assert_eq!(stream.byte_at(stream.consumed_pos()), Some(b';'));
  assert_eq!(stream.next(false).token.kind, TokenKind::Semicolon);
  assert_eq!(stream.next(false).token.kind, TokenKind::Eof);
}

#[test]
fn generic_value_fast_forward_buffers_plain_ascii_candidate_once() {
  let input = "red imported blue;";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);

  assert_eq!(stream.consumed_pos(), stream.lexer().scan_pos());
  stream.fast_forward_generic_value_if_buffer_empty(
    false,
    false,
    false,
    |ident| ident == "imported",
    |_| false,
  );
  assert_eq!(stream.consumed_pos(), 4);
  assert_eq!(stream.lexer().scan_pos(), 12);

  let candidate = stream.next(false).token;
  assert_eq!(candidate.kind, TokenKind::Ident);
  assert_eq!(candidate.range, Range::new(4, 12));
  assert_eq!(stream.consumed_pos(), 12);
  assert_eq!(stream.lexer().scan_pos(), 12);
}

#[test]
fn fast_forward_skips_plain_nesting() {
  let input = "foo(bar())";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  let open = stream.next(false).token;
  assert_eq!(open.kind, TokenKind::Function);
  assert_eq!(open.range, Range::new(0, 4));
  assert_eq!(
    stream.fast_forward(TokenKind::RightParenthesis),
    Some(Range::new(4, 10))
  );
  assert_eq!(stream.consumed_pos(), 10);
  assert_eq!(stream.lexer().scan_pos(), 10);
  assert_eq!(stream.next(false).token.kind, TokenKind::Eof);
}

#[test]
fn fast_forward_skips_mixed_brackets() {
  let input = "foo([a{b}])";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  let open = stream.next(false).token;
  assert_eq!(open.kind, TokenKind::Function);
  assert_eq!(
    stream.fast_forward(TokenKind::RightParenthesis),
    Some(Range::new(4, 11))
  );
  assert_eq!(stream.next(false).token.kind, TokenKind::Eof);
}

#[test]
fn fast_forward_ignores_brackets_inside_strings_and_comments() {
  let input = "foo(\")\" /* ) */ )";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  stream.next(false);
  assert_eq!(
    stream.fast_forward(TokenKind::RightParenthesis),
    Some(Range::new(4, 17))
  );
  assert_eq!(stream.next(false).token.kind, TokenKind::Eof);
}

#[test]
fn fast_forward_skips_escaped_closing_bracket() {
  let input = r"foo(a\)b)";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  stream.next(false);
  assert_eq!(
    stream.fast_forward(TokenKind::RightParenthesis),
    Some(Range::new(4, 9))
  );
  assert_eq!(stream.next(false).token.kind, TokenKind::Eof);
}

#[test]
fn fast_forward_returns_none_on_unclosed_function() {
  let input = "foo(bar";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  stream.next(false);
  assert_eq!(stream.fast_forward(TokenKind::RightParenthesis), None);
  assert_eq!(stream.lexer().scan_pos(), 4);
  assert_eq!(stream.consumed_pos(), 4);
  assert_eq!(stream.next(false).token.kind, TokenKind::Ident);
}

#[test]
fn fast_forward_returns_none_on_unbalanced_closing_bracket() {
  let input = "foo(]";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  stream.next(false);
  assert_eq!(stream.fast_forward(TokenKind::RightParenthesis), None);
  assert_eq!(stream.lexer().scan_pos(), 4);
  assert_eq!(stream.next(false).token.kind, TokenKind::RightSquareBracket);
}

#[test]
fn fast_forward_tracks_next_token_range() {
  let input = "foo(bar) baz";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  stream.next(false);
  assert_eq!(
    stream.fast_forward(TokenKind::RightParenthesis),
    Some(Range::new(4, 8))
  );
  let next = stream.next(false).token;
  assert_eq!(next.kind, TokenKind::Ident);
  assert_eq!(next.range, Range::new(9, 12));
}

#[test]
fn fast_forward_skips_non_ascii_boundaries() {
  let input = "foo(中)文)";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  stream.next(false);
  assert_eq!(
    stream.fast_forward(TokenKind::RightParenthesis),
    Some(Range::new(4, 8))
  );
  let next = stream.next(false).token;
  assert_eq!(next.kind, TokenKind::Ident);
  assert_eq!(next.range, Range::new(8, 11));
  assert_eq!(stream.next(false).token.kind, TokenKind::RightParenthesis);
}

#[test]
fn fast_forward_stops_at_top_level_semicolon() {
  let input = "supports(display: flex; display: grid)";
  let mut lexer = Lexer::new(input, ());
  let mut stream = TokenStream::from_lexer(&mut lexer);
  stream.next(false);
  assert_eq!(stream.fast_forward(TokenKind::RightParenthesis), None);
  assert_eq!(stream.lexer().scan_pos(), 9);
  loop {
    let token = stream.next(false).token;
    if token.kind == TokenKind::Semicolon {
      assert_eq!(token.range, Range::new(22, 23));
      break;
    }
  }
}

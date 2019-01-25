//! Testign for the AST parser

use super::*;
use crate::parsers::test::{create_span, error_context, error_list_contains};
use crate::source_location::{LocatedString, Location};

#[test]
fn simple() {
    let block = create_span("ab");
    let res = assert_ok!(parse_ast(block.span()));

    assert_complete!(res.0);
    assert_eq!(
        res.1,
        ast::constant(
            Location::test_location(1, 1),
            LocatedString::test_new(1, 1, "ab")
        )
    )
}

#[test]
fn single_char_ref() {
    let block = create_span("$a");
    let res = assert_ok!(parse_ast(block.span()));

    assert_complete!(res.0);
    assert_eq!(
        res.1,
        ast::variable_reference(
            Location::test_location(1, 1),
            ast::constant(
                Location::test_location(1, 2),
                LocatedString::test_new(1, 2, "a")
            )
        )
    )
}

#[test]
fn dollar_at_end() {
    let block = create_span("$");
    let res = assert_ok!(parse_ast(block.span()));

    assert_complete!(res.0);
    assert_eq!(
        res.1,
        ast::constant(
            Location::test_location(1, 1),
            LocatedString::test_new(1, 1, "$")
        )
    );
}

#[test]
fn dollar_escape() {
    let block = create_span("$$");
    let res = assert_ok!(parse_ast(block.span()));

    assert_complete!(res.0);
    assert_eq!(
        res.1,
        ast::constant(
            Location::test_location(1, 1),
            LocatedString::test_new(1, 2, "$")
        )
    );
}

#[test]
fn long_var_name() {
    let block = create_span("$(foo)");
    let res = assert_ok!(parse_ast(block.span()));

    assert_complete!(res.0);
    assert_eq!(
        res.1,
        ast::variable_reference(
            Location::test_location(1, 1),
            ast::constant(
                Location::test_location(1, 3),
                LocatedString::test_new(1, 3, "foo")
            )
        )
    );
}

#[test]
fn recursive_variable_expansion() {
    let block = create_span("$($(foo))");
    let res = assert_ok!(parse_ast(block.span()));

    assert_complete!(res.0);
    assert_eq!(
        res.1,
        ast::variable_reference(
            Location::test_location(1, 1),
            ast::variable_reference(
                Location::test_location(1, 3),
                ast::constant(
                    Location::test_location(1, 5),
                    LocatedString::test_new(1, 5, "foo")
                )
            )
        )
    );
}

#[test]
fn strip_basic() {
    let block = create_span("$(strip foo)");
    let res = assert_ok!(parse_ast(block.span()));
    assert_complete!(res.0);
    assert_eq!(
        res.1,
        ast::strip(
            Location::test_location(1, 3),
            ast::constant(
                Location::test_location(1, 9),
                LocatedString::test_new(1, 9, "foo")
            )
        )
    )
}

#[test]
fn strip_too_many_args() {
    let block = create_span("$(strip foo,extra)");
    let err = assert_err!(parse_ast(block.span()));
    assert_err_contains!(err, ParseErrorKind::ExtraArguments("strip"));
}

#[test]
fn words_basic() {
    let block = create_span("$(words foo bar)");
    let res = assert_ok!(parse_ast(block.span()));
    assert_complete!(res.0);
    assert_eq!(
        res.1,
        ast::words(
            Location::test_location(1, 3),
            ast::constant(
                Location::test_location(1, 9),
                LocatedString::test_new(1, 9, "foo bar")
            )
        )
    )
}

#[test]
fn word_basic() {
    let block = create_span("$(word 1,foo)");
    let res = assert_ok!(parse_ast(block.span()));
    assert_complete!(res.0);
    assert_eq!(
        res.1,
        ast::word(
            Location::test_location(1, 3),
            ast::constant(
                Location::test_location(1, 8),
                LocatedString::test_new(1, 8, "1")
            ),
            ast::constant(
                Location::test_location(1, 10),
                LocatedString::test_new(1, 10, "foo")
            )
        )
    )
}

#[test]
fn word_too_few_args() {
    let block = create_span("$(word 1)");
    let err = assert_err!(parse_ast(block.span()));
    assert_err_contains!(err, ParseErrorKind::InsufficientArguments("word"));
}

#[test]
fn word_too_many_args() {
    let block = create_span("$(word 1,foo,extra)");
    let err = assert_err!(parse_ast(block.span()));
    assert_err_contains!(err, ParseErrorKind::ExtraArguments("word"));
}

#[test]
fn argument_terminated_with_comma() {
    let block = create_span("foo,");
    let res = assert_ok!(function_argument(block.span()));

    assert_segments_eq!(res.0, [(",", Location::test_location(1, 4))]);
    assert_segments_eq!(res.1, [("foo", Location::test_location(1, 1))]);
}

#[test]
fn argument_terminated_with_eof() {
    let block = create_span("foo");
    let res = assert_ok!(function_argument(block.span()));

    assert_complete!(res.0);
    assert_segments_eq!(res.1, [("foo", Location::test_location(1, 1))]);
}

#[test]
fn argument_ignores_internal_commas() {
    let block = create_span("$(some_func a,b)");
    let res = assert_ok!(function_argument(block.span()));

    assert_complete!(res.0);
    assert_segments_eq!(res.1, [("$(some_func a,b)", Location::test_location(1, 1))]);

    let block = create_span("${some_func a,b}");
    let res = assert_ok!(function_argument(block.span()));

    assert_complete!(res.0);
    assert_segments_eq!(res.1, [("${some_func a,b}", Location::test_location(1, 1))]);
}

#[test]
fn unbalanced_reference() {
    let block = create_span("$(foo");
    let err = assert_err!(parse_ast(block.span()));
    assert_err_contains!(err, ParseErrorKind::UnternimatedVariable);
}
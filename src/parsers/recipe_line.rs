//! Parser for recipe lines

use super::{lift_collapsed_span_error, makefile_grab_line};
use crate::ast::AstNode;
use crate::evaluated::{Block, BlockSpan, ContentReference};
use crate::parsers::ast::parse_ast;
use crate::ParseErrorKind;
use nom::IResult;

/// Recognizes a recipe line
pub(super) fn recipe_line<'a>(
    i: BlockSpan<'a>,
    command_char: char,
) -> IResult<BlockSpan<'a>, AstNode, ParseErrorKind> {
    use nom::{InputIter, Slice};
    debug!("Get recipe from {:?}", i.into_string());

    let (i, (input_line, _)) = add_return_error!(
        i,
        ErrorKind::Custom(ParseErrorKind::RecipeExpected),
        preceded!(
            fix_error!(ParseErrorKind, char!(command_char)),
            makefile_grab_line
        )
    )?;

    let line_start = i;

    let mut fragments: Vec<ContentReference> = Vec::new();

    let it = input_line.iter_indices();
    let mut push_start = 0;
    let mut strip_next_if_cmd = false;
    for (idx, ch) in it {
        // XXX: This doesn't handle Windows newlines
        if ch == '\n' {
            fragments.push(input_line.slice(push_start..idx + 1).to_content_reference());
            strip_next_if_cmd = true;
            push_start = idx + 1;
        }
        if ch == '\t' && strip_next_if_cmd {
            push_start = idx + 1;
            strip_next_if_cmd = false;
        }
    }

    if push_start < input_line.len() {
        fragments.push(input_line.slice(push_start..).to_content_reference());
    }

    let line = Block::new(i.parent().raw_sensitivity(), fragments);

    let (_, ast) = parse_ast(line.span()).map_err(|e| lift_collapsed_span_error(e, line_start))?;

    Ok((i, ast))
}

#[cfg(test)]
mod test {
    use super::recipe_line;
    use crate::ast;
    use crate::parsers::test;
    use crate::parsers::test::create_span;
    use crate::source_location::{LocatedString, Location};
    use crate::ParseErrorKind;
    use nom::{error_to_list, ErrorKind};
    use pretty_assertions::assert_eq;

    macro_rules! recipe_line_test {
        ($span_contents:expr, $out_name:ident) => {
            crate::test::setup();

            let test_span = create_span($span_contents);
            let test_span = test_span.span();

            let $out_name = assert_ok!(recipe_line(test_span, '\t'));
        };
    }

    #[test]
    fn single_line() {
        recipe_line_test!("\ta\n", parse);
        assert_complete!(parse.0);
        assert_eq!(parse.1, ast::constant(LocatedString::test_new(1, 2, "a")));
    }

    #[test]
    fn at_eof() {
        recipe_line_test!("\ta", parse);
        assert_complete!(parse.0);
        assert_eq!(parse.1, ast::constant(LocatedString::test_new(1, 2, "a")));
    }

    #[test]
    fn multi_line_cont() {
        recipe_line_test!("\ta\\\n\tb", parse);
        assert_complete!(parse.0);
        assert_eq!(
            parse.1,
            ast::collapsing_concat(
                Location::test_location(1, 2),
                vec![
                    ast::constant(LocatedString::test_new(1, 2, "a\\\n")),
                    ast::constant(LocatedString::test_new(2, 2, "b"))
                ]
            )
        );
    }

    #[test]
    fn multi_line_not_cont() {
        recipe_line_test!("\ta\n\tb", parse);
        assert_segments_eq!(parse.0, [("\tb", Location::test_location(2, 1))]);
        assert_eq!(parse.1, ast::constant(LocatedString::test_new(1, 2, "a")));
    }

    #[test]
    fn multi_line_not_command() {
        recipe_line_test!("\ta\n\n", parse);
        assert_segments_eq!(parse.0, [("\n", Location::test_location(2, 1))]);
        assert_eq!(parse.1, ast::constant(LocatedString::test_new(1, 2, "a")));
    }

    #[test]
    fn multi_line_but_comment() {
        recipe_line_test!("\ta # \n\n", parse);
        assert_segments_eq!(parse.0, [("# \n\n", Location::test_location(1, 4))]);
        assert_eq!(parse.1, ast::constant(LocatedString::test_new(1, 2, "a ")));
    }

    #[test]
    fn collapse_parens_escapes() {
        recipe_line_test!("\techo '(a \\\\\\\\\n\t b)", parse);
        assert_complete!(parse.0);
        assert_eq!(
            parse.1,
            ast::collapsing_concat(
                Location::test_location(1, 2),
                vec![
                    ast::constant(LocatedString::test_new(1, 2, "echo '(a \\\\\\\\\n")),
                    ast::constant(LocatedString::test_new(2, 2, " b)"))
                ]
            )
        )
    }

    #[test]
    fn collapse_parens_no_function() {
        recipe_line_test!("\techo '(a \\\n\t b)", parse);
        assert_complete!(parse.0);
        assert_eq!(
            parse.1,
            ast::collapsing_concat(
                Location::test_location(1, 2),
                vec![
                    ast::constant(LocatedString::test_new(1, 2, "echo '(a \\\n")),
                    ast::constant(LocatedString::test_new(2, 2, " b)"))
                ]
            )
        );
    }

    #[test]
    fn collapse_in_function() {
        recipe_line_test!("\techo '$(if t,a \\\n\t b)", parse);
        assert_complete!(parse.0);
        assert_eq!(
            parse.1,
            ast::collapsing_concat(
                Location::test_location(1, 2),
                vec![
                    ast::constant(LocatedString::test_new(1, 2, "echo '")),
                    ast::if_fn(
                        Location::test_location(1, 10),
                        ast::constant(LocatedString::test_new(1, 13, "t")),
                        ast::constant(LocatedString::test_new(1, 15, "a")),
                        ast::constant(LocatedString::test_new(2, 3, "b"))
                    )
                ]
            )
        );
    }

    #[test]
    fn does_not_match_no_prefix() {
        let test_span = create_span("this is not a recipe line");
        let test_span = test_span.span();
        let parse = recipe_line(test_span, '\t');

        assert!(parse.is_err());
        let parse = parse.err().unwrap();
        assert!(match parse {
            nom::Err::Error(_) => true,
            _ => false,
        });
        let context = test::error_context(parse);
        assert!(context.is_some());
        let context = context.unwrap();
        let errors = error_to_list(&context);
        assert!(test::error_list_contains(
            &errors,
            ErrorKind::Custom(ParseErrorKind::RecipeExpected)
        ));
    }
}

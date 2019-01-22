//! Parser for recipe lines

use super::{ends_with_backslash, makefile_grab_line};
use crate::evaluated::{Block, BlockSpan, ContentReference};
use crate::ParseErrorKind;
use nom::IResult;
use std::sync::Arc;

/// Recognizes a recipe line
pub(super) fn recipe_line<'a>(
    i: BlockSpan<'a>,
    command_char: char,
) -> IResult<BlockSpan<'a>, Arc<Block>, ParseErrorKind> {
    macro_rules! grab_command_line(
        ($i:expr ) => { grab_command_line!($i , ) };
        ($i:expr, ) => {
            preceded!(
                $i,
                fix_error!(ParseErrorKind, char!(command_char)),
                makefile_grab_line
        )};
    );

    // Grab the first command line
    let (mut i, (line, end_reason)) = add_return_error!(
        i,
        ErrorKind::Custom(ParseErrorKind::RecipeExpected),
        grab_command_line!()
    )?;
    let mut line = line.to_new_block();
    eprintln!("{:?}", line.into_string());

    // If the line has a comment, stop the line (and processing)
    if end_reason == super::LineEndReason::Comment {
        return Ok((i, line));
    }

    while ends_with_backslash(line.span()) & 1 == 1 {
        Arc::make_mut(&mut line).push(ContentReference::newline()); // Add back the \n we striped
        let next_line_result = grab_command_line!(i);
        match next_line_result {
            Ok((new_i, (next_line, end_reason))) => {
                Arc::make_mut(&mut line).push(next_line.to_content_reference());
                i = new_i;

                // If the line has a comment, stop the line (and processing)
                if end_reason == super::LineEndReason::Comment {
                    return Ok((i, line));
                }
            }
            Err(_) => {
                // If we couldn't grab a line, this must not be a command line
                break;
            }
        }
    }

    Ok((i, line))
}

#[cfg(test)]
mod test {
    use super::recipe_line;
    use crate::parsers::test;
    use crate::parsers::test::create_span;
    use crate::source_location::Location;
    use crate::ParseErrorKind;
    use nom::{error_to_list, ErrorKind};

    #[test]
    fn single_line() {
        let test_span = create_span("\ta\n");
        let test_span = test_span.span();
        let parse = assert_ok!(recipe_line(test_span, '\t'));
        assert_complete!(parse.0);
        assert_segments_eq!(parse.1.span(), [("a", Location::test_location(1, 2))])
    }

    #[test]
    fn at_eof() {
        let test_span = create_span("\ta");
        let test_span = test_span.span();
        let parse = assert_ok!(recipe_line(test_span, '\t'));
        assert_complete!(parse.0);
        assert_segments_eq!(parse.1.span(), [("a", Location::test_location(1, 2))]);
    }

    #[test]
    fn multi_line_cont() {
        let test_span = create_span("\ta\\\n\tb");
        let test_span = test_span.span();
        let parse = assert_ok!(recipe_line(test_span, '\t'));
        assert_complete!(parse.0);
        assert_segments_eq!(
            parse.1.span(),
            [
                ("a\\", Location::test_location(1, 2)),
                ("\n", Location::Synthetic),
                ("b", Location::test_location(2, 2))
            ]
        );
    }

    #[test]
    fn multi_line_not_cont() {
        let test_span = create_span("\ta\n\tb");
        let test_span = test_span.span();
        let parse = assert_ok!(recipe_line(test_span, '\t'));
        assert_segments_eq!(parse.0, [("\tb", Location::test_location(2, 1))]);
        assert_segments_eq!(parse.1.span(), [("a", Location::test_location(1, 2))]);
    }

    #[test]
    fn multi_line_not_command() {
        let test_span = create_span("\ta\n\n");
        let test_span = test_span.span();
        let parse = assert_ok!(recipe_line(test_span, '\t'));
        assert_segments_eq!(parse.0, [("\n", Location::test_location(2, 1))]);
        assert_segments_eq!(parse.1.span(), [("a", Location::test_location(1, 2))]);
    }

    #[test]
    fn multi_line_but_comment() {
        let test_span = create_span("\ta # \n\n");
        let test_span = test_span.span();
        let parse = assert_ok!(recipe_line(test_span, '\t'));
        assert_segments_eq!(parse.0, [("# \n\n", Location::test_location(1, 4))]);
        assert_segments_eq!(parse.1.span(), [("a ", Location::test_location(1, 2))]);
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
        // assert_eq!(parse, (leftover_span("a", 32, 2), ()));
    }
}

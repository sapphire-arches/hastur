use super::*;
use crate::evaluated::test::single_block;
use crate::parsers::{makefile_line, ParserCompliance};

mod slice {
    use super::*;
    #[test]
    fn no_skip() {
        let block = single_block("a\\\nbc\\\nd");
        let span = block.span();
        let under_test = makefile_line(span, ParserCompliance::GNU, false).unwrap().1;
        let under_test = under_test.span();
        let res = under_test.slice(0..1);

        assert_eq!(res.into_string(), "a");
    }

    #[test]
    fn skip_then_contained() {
        let block = single_block("a\\\nbc\\\nd");
        let span = block.span();
        let under_test = makefile_line(span, ParserCompliance::GNU, false).unwrap().1;
        let under_test = under_test.span();
        let res = under_test.slice(2..4);

        assert_eq!(res.into_string(), "bc");
    }

    #[test]
    fn skip_then_split() {
        let block = single_block("a\\\nbc\\\nd");
        let span = block.span();
        let under_test = makefile_line(span, ParserCompliance::GNU, false).unwrap().1;
        let under_test = under_test.span();
        let res = under_test.slice(2..5);

        assert_eq!(res.into_string(), "bc ");
    }

    #[test]
    fn skip_then_rest() {
        let block = single_block("a\\\nbc\\\nd");
        let span = block.span();
        let under_test = makefile_line(span, ParserCompliance::GNU, false).unwrap().1;
        let under_test = under_test.span();
        eprintln!("{:?}", under_test.contents);
        let res = under_test.slice(3..);
        eprintln!("{:?}", res.contents);

        assert_eq!(res.into_string(), String::from("c d"));
    }

    #[test]
    fn skip_then_rest_offset() {
        let block = single_block("ab\\\nbc\\\nd");
        let span = block.span();
        let under_test = makefile_line(span, ParserCompliance::GNU, false).unwrap().1;
        let mut under_test = under_test.span();
        eprintln!("{:?}", under_test.into_string());
        under_test.offset += 1;
        under_test.revalidate_offset();
        let res = under_test.slice(3..);

        assert_eq!(res.into_string(), String::from("c d"));
    }

    #[test]
    fn range_to_simple() {
        let under_test = single_block("a\\\nbcdefg");
        let under_test = makefile_line(under_test.span(), ParserCompliance::GNU, false)
            .unwrap()
            .1;
        let under_test = under_test.span();

        assert_eq!(under_test.slice(..1).into_string(), "a");
        assert_eq!(under_test.slice(..3).into_string(), "a b");
    }
}

mod split_at {
    use super::*;

    #[test]
    fn simple() {
        let under_test = single_block("a\\\n1\\\nc");
        let under_test = makefile_line(under_test.span(), ParserCompliance::GNU, false)
            .unwrap()
            .1;
        let under_test = under_test.span();
        let res = under_test.split_at_position(|c| c == '1');
        let res = res.ok().unwrap();

        assert_eq!(res.0.into_string(), "1 c");
        assert_eq!(res.1.into_string(), "a ");
    }

    #[test]
    fn simple1() {
        let under_test = single_block("a\\\n1\\\nc");
        let under_test = makefile_line(under_test.span(), ParserCompliance::GNU, false)
            .unwrap()
            .1;
        let under_test = under_test.span();
        let res = under_test.split_at_position1(|c| c == '1', ErrorKind::Char);
        let res = res.ok().unwrap();

        assert_eq!(res.0.into_string(), "1 c");
        assert_eq!(res.1.into_string(), "a ");
    }

    #[test]
    fn atleast_fails() {
        let under_test = single_block("a\\\n1\\\nc");
        let under_test = under_test.span();
        let res = under_test.split_at_position1(|c| c == 'a', ErrorKind::Char);

        assert!(res.is_err());
    }
}

mod compare {
    use super::*;

    #[test]
    fn simple_eq() {
        let under_test = single_block("asdf");
        let under_test = under_test.span();

        assert_eq!(under_test.compare("as"), CompareResult::Ok)
    }

    #[test]
    fn simple_same() {
        let under_test = single_block("asdf");
        let under_test = under_test.span();

        assert_eq!(under_test.compare("asdf"), CompareResult::Ok)
    }

    #[test]
    fn offset_eq() {
        let under_test = single_block("abcd");
        let under_test = under_test.span().slice(2..);

        assert_eq!(under_test.compare("cd"), CompareResult::Ok);
    }

    #[test]
    fn simple_incomplete() {
        let under_test = single_block("asdf");
        let under_test = under_test.span();

        assert_eq!(under_test.compare("asdfjkl"), CompareResult::Incomplete)
    }

    #[test]
    fn simple_error() {
        let under_test = single_block("asdf");
        let under_test = under_test.span();

        assert_eq!(under_test.compare("asdq"), CompareResult::Error)
    }

    #[test]
    fn no_case_eq() {
        let under_test = single_block("asdf");
        let under_test = under_test.span();

        assert_eq!(under_test.compare_no_case("aS"), CompareResult::Ok)
    }

    #[test]
    fn no_caes_same() {
        let under_test = single_block("asdf");
        let under_test = under_test.span();

        assert_eq!(under_test.compare_no_case("aSdf"), CompareResult::Ok)
    }

    #[test]
    fn no_case_incomplete() {
        let under_test = single_block("asdf");
        let under_test = under_test.span();

        assert_eq!(
            under_test.compare_no_case("asDfjkl"),
            CompareResult::Incomplete
        )
    }

    #[test]
    fn no_case_error() {
        let under_test = single_block("asdf");
        let under_test = under_test.span();

        assert_eq!(under_test.compare_no_case("asDq"), CompareResult::Error)
    }
}

mod input_iter {
    use super::*;

    #[test]
    fn elements_iter() {
        let block = single_block("a\\\nbc\\\nd");
        let span = block.span();
        let under_test = makefile_line(span, ParserCompliance::GNU, false).unwrap().1;
        let under_test = under_test.span();
        eprintln!("{:?}", under_test.into_string());
        let mut iter = under_test.iter_elements();

        assert_eq!(iter.next(), Some('a'));
        assert_eq!(iter.next(), Some(' '));
        assert_eq!(iter.next(), Some('b'));
        assert_eq!(iter.next(), Some('c'));
        assert_eq!(iter.next(), Some(' '));
        assert_eq!(iter.next(), Some('d'));
    }

    #[test]
    fn elments_iter_offset() {
        let block = single_block("ab\\\nb");
        let under_test = makefile_line(block.span(), ParserCompliance::GNU, false)
            .unwrap()
            .1;
        let mut under_test = under_test.span();
        eprintln!("Span under test is {:?}", under_test.into_string());
        under_test.offset += 1;
        under_test.revalidate_offset();
        let mut iter = under_test.iter_elements();

        assert_eq!(iter.next(), Some('b'));
        assert_eq!(iter.next(), Some(' '));
        assert_eq!(iter.next(), Some('b'));
    }
}

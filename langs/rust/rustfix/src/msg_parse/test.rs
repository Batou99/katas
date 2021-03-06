//  move tags and errors to different files
//  check fields position
//  test external to module
//  test more than one message


use msg_parse::{ParsingResult, Parsing, add_char, errors, ParsingState};

use test_diff;


pub const MSG_TEST_WIKIPEDIA1: &'static str = "8=FIX.4.2|9=178|35=8|49=PHLX|56=PERS|52=20071123-05:\
                                               30:00.000|11=ATOMNOCCC9990900|20=3|150=E|39=E|55=MSFT|167=CS|54=1|38=15|40=2|44=15|58=PHLX \
                                               EQUITY TESTING|59=0|47=C|32=0|31=0|151=15|14=0|6=0|10=128|";

pub const MSG_TEST_WIKIPEDIA2: &'static str = "8=FIX.4.2|9=65|35=A|49=SERVER|56=CLIENT|34=177|52=20090107-18:\
                                               15:16|98=0|108=30|10=062|";

pub const MSG_TEST: &'static str = "8=FIX.4.4|9=122|35=D|34=215|49=CLIENT12|52=20100225-19:41:57.\
                                    316|56=B|1=Marcel|11=13346|21=1|40=2|44=5|54=1|59=0|60=20100225-19:\
                                    39:52.020|10=072|";



macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
    }

macro_rules! btree {
    ($( $key: expr => $val: expr ),*) => {{
         let mut result = ::std::collections::BTreeMap::new();
         $( result.insert($key, $val); )*
         result
    }}
    }



fn add_char_incomplete(mut parser: Parsing, ch: char) -> Parsing {
    match add_char(parser, ch) {
        ParsingResult::Parsing(parsing) => parser = parsing,
        ParsingResult::ParsedOK(_) => panic!("ParsedOK in icomplete message"),
        ParsingResult::ParsedErrors { parsed: _, errors: _ } => {
            panic!("ParsedErrors on incomplete message")
        }
    }
    parser
}

fn conv_separator(ch: char) -> char {
    if ch == '^' || ch == '|' {
        1u8 as char
    } else {
        ch
    }
}

fn add_chars_incomplete(mut parser: Parsing, s: &'static str) -> Parsing {
    for ch in s.chars() {
        parser = add_char_incomplete(parser, conv_separator(ch));
    }
    parser
}

fn add_chars_full_message(mut parser: Parsing, s: &'static str) -> ParsingResult {
    for ch in s.chars() {
        match add_char(parser, conv_separator(ch)) {
            ParsingResult::Parsing(parsing) => parser = parsing,
            ParsingResult::ParsedErrors { parsed: p, errors: e } => {
                return ParsingResult::ParsedErrors {
                    parsed: p,
                    errors: e,
                };
            }
            ParsingResult::ParsedOK(parsed) => {
                return ParsingResult::ParsedOK(parsed);
            }
        }
    }
    panic!("No full message processed")
}





#[test]
fn init_add_char() {
    let mut parser = Parsing::new();

    parser = add_char_incomplete(parser, '1');

    ass_eqdf!{
        parser.parsed.orig_msg => "1".to_string(),
        parser.parsed.msg_length => 1,
        parser.reading_tag => 1
    };
}


#[test]
fn invalid_first_char() {
    let mut parser = Parsing::new();
    parser = add_char_incomplete(parser, 'a');

    ass_eqdf!{
        parser.parsed.orig_msg => "a".to_string(),
        parser.parsed.msg_length => 1,
        parser.reading_tag => 0
    };
}

#[test]
fn invalid_second_char() {
    let mut parser = Parsing::new();
    parser = add_char_incomplete(parser, '1');
    parser = add_char_incomplete(parser, 'a');

    ass_eqdf!{
        parser.parsed.orig_msg => "1a".to_string(),
        parser.parsed.msg_length => 2,
        parser.reading_tag => 1
    };
}


#[test]
fn invalid_chars_2errors() {
    let mut parser = Parsing::new();
    parser = add_char_incomplete(parser, '1');
    parser = add_char_incomplete(parser, 'a');
    parser = add_char_incomplete(parser, 'b');

    ass_eqdf!{
        parser.parsed.orig_msg => "1ab".to_string(),
        parser.parsed.msg_length => 3,
        parser.reading_tag => 1,
        parser.current_field_error =>
                    Some((2, errors::TAG_INVALID_CHAR))
    };
}

#[test]
fn invalid_chars_2errors_andvalids() {
    let mut parser = Parsing::new();
    parser = add_chars_incomplete(parser, "12ab34");

    ass_eqdf!{
        parser.parsed.orig_msg => "12ab34".to_string(),
        parser.parsed.msg_length => 6,
        parser.reading_tag => 12,
        parser.current_field_error =>
                    Some((3, errors::TAG_INVALID_CHAR))
    };
}


#[test]
fn invalid_chars_2errors_and_valids_non_consecutives() {
    let mut parser = Parsing::new();
    parser = add_chars_incomplete(parser, "12a3b45");

    ass_eqdf!{
        parser.parsed.orig_msg => "12a3b45".to_string(),
        parser.parsed.msg_length => 7,
        parser.reading_tag => 12,
        parser.current_field_error =>
                    Some((3, errors::TAG_INVALID_CHAR))
    };
}


//  too long
//      error and igonre big tag

#[test]
fn too_long_tag() {
    let mut parser = Parsing::new();
    parser = add_chars_incomplete(parser, "1234567890");

    ass_eqdf!{
        parser.parsed.orig_msg => "1234567890".to_string(),
        parser.parsed.msg_length =>  10,
        parser.reading_tag =>  123_456_7,
        parser.current_field_error => Some((7, errors::TAG_TOO_LONG))
    };
}


#[test]
fn too_long_tag_ignore_excess() {
    let mut parser = Parsing::new();
    parser = add_chars_incomplete(parser, "123456789012345");

    ass_eqdf!{
        parser.parsed.orig_msg => "123456789012345".to_string(),
        parser.parsed.msg_length => 15,
        parser.reading_tag => 123_456_7,
        parser.current_field_error => Some((7, errors::TAG_TOO_LONG))
    };
}



//  =
//      start receiving val
#[test]
fn finish_tag() {
    let mut parser = Parsing::new();
    parser = add_chars_incomplete(parser, "123456=");

    ass_eqdf!{
        parser.parsed.orig_msg => "123456=".to_string(),
        parser.parsed.msg_length=>  7,
        parser.reading_tag => 123_456,
        parser.state => ParsingState::StReadingValue
    };
}




//  receiving val
//      "a"
#[test]
fn reading_val() {
    let mut parser = Parsing::new();
    parser = add_chars_incomplete(parser, "123456=a");

    ass_eqdf!{
        parser.parsed.orig_msg => "123456=a".to_string(),
        parser.parsed.msg_length => 8,
        parser.reading_tag => 123_456,
        parser.reading_val => "a".to_string(),
        parser.state => ParsingState::StReadingValue
    };
}


//      "abcdefg"
#[test]
fn reading_val2() {
    let mut parser = Parsing::new();
    parser = add_chars_incomplete(parser, "123456=abcdefg");

    ass_eqdf!{
        parser.parsed.orig_msg => "123456=abcdefg".to_string(),
        parser.parsed.msg_length => 14,
        parser.reading_tag => 123_456,
        parser.reading_val => "abcdefg".to_string(),
        parser.state => ParsingState::StReadingValue
    };
}


//      too long val received
#[test]
fn too_long_val() {
    let mut parser = Parsing::new();
    parser = add_chars_incomplete(parser,
                                  "123456=abcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefg");

    ass_eqdf!{
        parser.parsed.orig_msg =>
            "123456=abcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefg"
            .to_string(),
        parser.parsed.msg_length => 77,
        parser.reading_tag => 123_456,
        parser.reading_val => "abcdefgabcdefgabcdefgabcdefgabcdefgabcdefgabcdefga".to_string(),
        parser.state => ParsingState::StReadingValue,
        parser.current_field_error => Some((57, errors::VAL_TOO_LONG))
    };
}


//  received field  0x01
//      insert in map
#[test]
fn complete_field() {
    let mut parser = Parsing::new();
    parser = add_chars_incomplete(parser, "123456=abcdefg\u{01}");


    ass_eqdf!{
        parser.parsed.msg_map => btree![123456 => "abcdefg".to_string()],
        parser.parsed.orig_msg => "123456=abcdefg|".to_string(),
        parser.parsed.msg_length => 15,
        parser.reading_tag => 0,
        parser.reading_val => "".to_string(),
        parser.state => ParsingState::StReadingTag
    };
}



//  completed two fields
#[test]
fn complete_2field() {
    let mut parser = Parsing::new();
    parser = add_chars_incomplete(parser, "123456=abcdefg\u{01}123457=hijklmno\u{01}");


    ass_eqdf!{
        parser.parsed.msg_map =>
            btree![
                    123456 => "abcdefg".to_string(),
                    123457 => "hijklmno".to_string()
            ],
        parser.parsed.orig_msg => "123456=abcdefg|123457=hijklmno|".to_string(),
        parser.parsed.msg_length => 31,
        parser.reading_tag => 0,
        parser.reading_val => "".to_string(),
        parser.state => ParsingState::StReadingTag
    };
}



// check position fields
//  ...



//  completed field with two errors



//  received field  0x01 ERROR
//      at the beginning of tag
//      reading tag
//      after =






//  detected end of message
//      finished status
//      check message body length
//      check original message
//      check checksum
#[test]
fn full_message() {
    struct Checks {
        message: &'static str,
        body_length: usize,
        check_sum: u16,
    };

    let check_message = |c: Checks| {
        let parser = Parsing::new();

        let parsed = match add_chars_full_message(parser, c.message) {
            ParsingResult::ParsedOK(parsed) => parsed,
            ParsingResult::Parsing(_) => panic!("Incomplete parsing on full message"),
            ParsingResult::ParsedErrors { parsed: _, errors: _ } => {
                panic!("Incomplete parsing on full message")
            }
        };

        ass_eqdf!{
            parsed.body_length => c.body_length,
            parsed.orig_msg => c.message,
            parsed.check_sum => c.check_sum
        };
    };


    check_message(Checks {
        message: MSG_TEST_WIKIPEDIA1,
        body_length: 178,
        check_sum: 128,
    });
    check_message(Checks {
        message: MSG_TEST_WIKIPEDIA2,
        body_length: 65,
        check_sum: 62,
    });
    check_message(Checks {
        message: MSG_TEST,
        body_length: 122,
        check_sum: 72,
    });
}




// //  detected end of message 3 consecutives
// //      finished status
// //      check message length
// //      check original message
// //      check checksum
// #[test]
// fn full_messages3_consecutives() {
//     struct Checks {
//         message: &'static str,
//         body_length: usize,
//         check_sum: u16,
//     };

//     let check_message = |mut parser, c: Checks| {
//         parser = add_chars(parser, c.message);

//         ass_eqdf!{
//             parser.parsed.body_length => c.body_length,
//             parser.reading_tag => 0,
//             parser.reading_val => "".to_string(),
//             parser.state => ParsingState::StFinished,
//             parser.parsed.orig_msg => c.message,
//             parser.parsed.check_sum => c.check_sum
//         };
//         parser
//     };


//     let mut parser = Parsing::new();
//     parser = check_message(parser,
//                            Checks {
//                                message: MSG_TEST_WIKIPEDIA1,
//                                body_length: 178,
//                                check_sum: 128,
//                            });
//     parser = check_message(parser,
//                            Checks {
//                                message: MSG_TEST_WIKIPEDIA2,
//                                body_length: 65,
//                                check_sum: 62,
//                            });
//     check_message(parser,
//                   Checks {
//                       message: MSG_TEST,
//                       body_length: 122,
//                       check_sum: 72,
//                   });
// }




//  process 3 messages, errors in second one

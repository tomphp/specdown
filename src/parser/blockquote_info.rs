use std::fmt;

use nom::{
    bytes::streaming::{tag, take_until},
    combinator::map,
    sequence::tuple,
};

use crate::parser::function_string;
use crate::types::{ScriptName, Source, Stream};

#[derive(Debug, PartialEq)]
pub enum BlockQuoteTypes {
    Script(ScriptName),
    Verify(Source),
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ParserFailed(String),
    UnknownFunction(String),
    MissingArgument(String, String),
    IncorrectArgumentType { expected: String, got: String },
    InvalidArgumentValue { got: String, expected: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ParserFailed(msg) => write!(f, "The parser failed: {}", msg),
            Self::UnknownFunction(name) => write!(f, "Unknown function: {}", name),
            Self::MissingArgument(func, arg) => {
                write!(f, "Function {} requires argument {}", func, arg)
            }
            Self::IncorrectArgumentType { expected, got } => write!(
                f,
                "Invalid argument type. Expected {}, got {}",
                expected, got
            ),
            Self::InvalidArgumentValue { got, expected } => write!(
                f,
                "Invalid argument value. Expected {}, got {}",
                expected, got
            ),
        }
    }
}

pub fn parse(input: &str) -> Result<BlockQuoteTypes, Error> {
    let p = tuple((take_until(","), tag(","), function_string::parse));
    let p = map(p, |(_language, _comma, func)| func);

    match p(input) {
        Ok((_, func)) => to_blockquote_type(&func),
        Err(nom_error) => Err(Error::ParserFailed(nom_error.to_string())),
    }
}

fn to_blockquote_type(f: &function_string::Function) -> Result<BlockQuoteTypes, Error> {
    match &f.name[..] {
        "script" => {
            let name = get_string_argument(&f, "name")?;
            Ok(BlockQuoteTypes::Script(ScriptName(name)))
        }
        "verify" => {
            let name = ScriptName(get_string_argument(&f, "script_name")?);
            let stream = to_stream(&get_token_argument(&f, "stream")?)?;
            Ok(BlockQuoteTypes::Verify(Source { name, stream }))
        }
        _ => Err(Error::UnknownFunction(f.name.clone())),
    }
}

fn to_stream(stream_name: &str) -> Result<Stream, Error> {
    match stream_name {
        "output" => Ok(Stream::Output),
        "stdout" => Ok(Stream::StdOut),
        "stderr" => Ok(Stream::StdErr),
        _ => Err(Error::InvalidArgumentValue {
            got: stream_name.to_string(),
            expected: "output, stdout or stderr".to_string(),
        }),
    }
}

fn get_string_argument(f: &function_string::Function, name: &str) -> Result<String, Error> {
    let arg = f
        .arguments
        .get(name)
        .ok_or_else(|| Error::MissingArgument(f.name.clone(), name.to_string()))?;

    match arg {
        function_string::ArgumentValue::String(s) => Ok(s.clone()),
        function_string::ArgumentValue::Token(_) => Err(Error::IncorrectArgumentType {
            expected: "string".to_string(),
            got: "token".to_string(),
        }),
    }
}

fn get_token_argument(f: &function_string::Function, name: &str) -> Result<String, Error> {
    let arg = f
        .arguments
        .get(name)
        .ok_or_else(|| Error::MissingArgument(f.name.clone(), name.to_string()))?;

    match arg {
        function_string::ArgumentValue::Token(t) => Ok(t.clone()),
        function_string::ArgumentValue::String(_) => Err(Error::IncorrectArgumentType {
            expected: "token".to_string(),
            got: "string".to_string(),
        }),
    }
}

mod tests {
    #[cfg(test)]
    use super::*;

    mod error {
        #[cfg(test)]
        use super::*;

        #[test]
        fn display_parser_failed() {
            assert_eq!(
                format!("{}", Error::ParserFailed("reason".to_string())),
                "The parser failed: reason"
            )
        }

        #[test]
        fn display_unknown_function() {
            assert_eq!(
                format!("{}", Error::UnknownFunction("funcy".to_string())),
                "Unknown function: funcy"
            )
        }

        #[test]
        fn display_missing_argument() {
            assert_eq!(
                format!(
                    "{}",
                    Error::MissingArgument("funcy".to_string(), "argy".to_string())
                ),
                "Function funcy requires argument argy"
            )
        }

        #[test]
        fn display_incorrect_argument_type() {
            assert_eq!(
                format!(
                    "{}",
                    Error::IncorrectArgumentType {
                        expected: "token".to_string(),
                        got: "string".to_string()
                    }
                ),
                "Invalid argument type. Expected token, got string"
            )
        }

        #[test]
        fn display_invalid_argument_value() {
            assert_eq!(
                format!(
                    "{}",
                    Error::InvalidArgumentValue {
                        expected: "true or false".to_string(),
                        got: "maybe".to_string()
                    }
                ),
                "Invalid argument value. Expected true or false, got maybe"
            )
        }
    }

    mod parse {
        #[cfg(test)]
        use super::*;

        mod script {
            #[cfg(test)]
            use super::*;

            #[test]
            fn succeeds_when_function_is_script() {
                let result = parse("shell,script(name=\"example-script\")");
                assert_eq!(
                    result,
                    Ok(BlockQuoteTypes::Script(ScriptName(
                        "example-script".to_string()
                    )))
                )
            }

            #[test]
            fn fails_when_name_is_missing() {
                let result = parse("shell,script()");
                assert_eq!(
                    result,
                    Err(Error::MissingArgument(
                        "script".to_string(),
                        "name".to_string()
                    ))
                )
            }
        }

        mod verify {
            #[cfg(test)]
            use super::*;

            #[test]
            fn succeeds_when_function_is_verify_and_stream_is_output() {
                let result = parse(",verify(script_name=\"example-script\", stream=output)");
                assert_eq!(
                    result,
                    Ok(BlockQuoteTypes::Verify(Source {
                        name: ScriptName("example-script".to_string()),
                        stream: Stream::Output
                    }))
                )
            }

            #[test]
            fn succeeds_when_function_is_verify_and_stream_is_stdout() {
                let result = parse(",verify(script_name=\"example-script\", stream=stdout)");
                assert_eq!(
                    result,
                    Ok(BlockQuoteTypes::Verify(Source {
                        name: ScriptName("example-script".to_string()),
                        stream: Stream::StdOut
                    }))
                )
            }

            #[test]
            fn succeeds_when_function_is_verify_and_stream_is_stderr() {
                let result = parse(",verify(script_name=\"example-script\", stream=stderr)");
                assert_eq!(
                    result,
                    Ok(BlockQuoteTypes::Verify(Source {
                        name: ScriptName("example-script".to_string()),
                        stream: Stream::StdErr
                    }))
                )
            }

            #[test]
            fn fails_when_function_is_verify_and_stream_is_unknown() {
                let result = parse(",verify(script_name=\"example-script\", stream=unknown)");
                assert_eq!(
                    result,
                    Err(Error::InvalidArgumentValue {
                        got: "unknown".to_string(),
                        expected: "output, stdout or stderr".to_string()
                    })
                )
            }

            #[test]
            fn fails_when_script_name_is_missing() {
                let result = parse("shell,verify(stream=stderr)");
                assert_eq!(
                    result,
                    Err(Error::MissingArgument(
                        "verify".to_string(),
                        "script_name".to_string()
                    ))
                )
            }

            #[test]
            fn fails_when_stream_is_missing() {
                let result = parse("shell,verify(script_name=\"the-script\")");
                assert_eq!(
                    result,
                    Err(Error::MissingArgument(
                        "verify".to_string(),
                        "stream".to_string()
                    ))
                )
            }
        }
    }
}

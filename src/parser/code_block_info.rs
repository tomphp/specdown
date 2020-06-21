use nom::{
    bytes::streaming::{tag, take_until},
    combinator::map,
    sequence::tuple,
};

use super::error::{Error, Result};
use super::function_string;
use crate::types::{ExitCode, FilePath, ScriptName, Source, Stream};

#[derive(Debug, PartialEq)]
pub enum CodeBlockType {
    Script(ScriptName, Option<ExitCode>),
    Verify(Source),
    CreateFile(FilePath),
}

pub fn parse(input: &str) -> Result<CodeBlockType> {
    let p = tuple((take_until(","), tag(","), function_string::parse));
    let p = map(p, |(_language, _comma, func)| func);

    match p(input) {
        Ok((_, func)) => to_code_block_type(&func),
        Err(nom_error) => Err(Error::ParserFailed(nom_error.to_string())),
    }
}

fn to_code_block_type(f: &function_string::Function) -> Result<CodeBlockType> {
    match &f.name[..] {
        "script" => script_to_code_block_type(f),
        "verify" => verify_to_code_block_type(f),
        "file" => file_to_code_block_type(f),
        _ => Err(Error::UnknownFunction(f.name.clone())),
    }
}

fn script_to_code_block_type(f: &function_string::Function) -> Result<CodeBlockType> {
    let name = get_string_argument(&f, "name")?;
    let expected_exit_code = if f.has_argument("expected_exit_code") {
        Some(ExitCode(get_integer_argument(f, "expected_exit_code")?))
    } else {
        None
    };
    Ok(CodeBlockType::Script(ScriptName(name), expected_exit_code))
}

fn file_to_code_block_type(f: &function_string::Function) -> Result<CodeBlockType> {
    let path = get_string_argument(&f, "path")?;
    Ok(CodeBlockType::CreateFile(FilePath(path)))
}

fn verify_to_code_block_type(f: &function_string::Function) -> Result<CodeBlockType> {
    let name = ScriptName(get_string_argument(&f, "script_name")?);
    let stream_name = get_token_argument(&f, "stream")?;
    let stream = to_stream(&stream_name).ok_or_else(|| Error::InvalidArgumentValue {
        function: f.name.to_string(),
        argument: "stream".to_string(),
        got: stream_name.to_string(),
        expected: "output, stdout or stderr".to_string(),
    })?;
    Ok(CodeBlockType::Verify(Source { name, stream }))
}

fn to_stream(stream_name: &str) -> Option<Stream> {
    match stream_name {
        "stdout" => Some(Stream::StdOut),
        "stderr" => Some(Stream::StdErr),
        _ => None,
    }
}

fn get_integer_argument(f: &function_string::Function, name: &str) -> Result<u32> {
    use function_string::ArgumentValue;

    match get_required_argument(f, name)? {
        ArgumentValue::Integer(num) => Ok(*num),
        ArgumentValue::String(_) => incorrect_argument_type_error(f, name, "integer", "string"),
        ArgumentValue::Token(_) => incorrect_argument_type_error(f, name, "integer", "token"),
    }
}

fn get_string_argument(f: &function_string::Function, name: &str) -> Result<String> {
    use function_string::ArgumentValue;

    match get_required_argument(f, name)? {
        ArgumentValue::String(s) => Ok(s.clone()),
        ArgumentValue::Integer(_) => incorrect_argument_type_error(f, name, "string", "integer"),
        ArgumentValue::Token(_) => incorrect_argument_type_error(f, name, "string", "token"),
    }
}

fn get_token_argument(f: &function_string::Function, name: &str) -> Result<String> {
    use function_string::ArgumentValue;

    match get_required_argument(f, name)? {
        ArgumentValue::Token(t) => Ok(t.clone()),
        ArgumentValue::Integer(_) => incorrect_argument_type_error(f, name, "token", "integer"),
        ArgumentValue::String(_) => incorrect_argument_type_error(f, name, "token", "string"),
    }
}

fn get_required_argument<'a>(
    f: &'a function_string::Function,
    name: &str,
) -> Result<&'a function_string::ArgumentValue> {
    f.arguments.get(name).ok_or_else(|| Error::MissingArgument {
        function: f.name.clone(),
        argument: name.to_string(),
    })
}

fn incorrect_argument_type_error<T>(
    f: &function_string::Function,
    argument: &str,
    expected: &str,
    got: &str,
) -> Result<T> {
    Err(Error::IncorrectArgumentType {
        function: f.name.to_string(),
        argument: argument.to_string(),
        expected: expected.to_string(),
        got: got.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::{parse, CodeBlockType, Error, ExitCode, FilePath, ScriptName, Source, Stream};

    mod parse {
        use super::{parse, CodeBlockType, Error, ExitCode, FilePath, ScriptName, Source, Stream};

        mod script {
            use super::{parse, CodeBlockType, Error, ExitCode, ScriptName};

            #[test]
            fn succeeds_when_function_is_script() {
                let result = parse("shell,script(name=\"example-script\")");
                assert_eq!(
                    result,
                    Ok(CodeBlockType::Script(
                        ScriptName("example-script".to_string(),),
                        None
                    ))
                )
            }

            #[test]
            fn succeeds_when_function_is_script_with_expected_exit_code() {
                let result = parse("shell,script(name=\"example-script\", expected_exit_code=2)");
                assert_eq!(
                    result,
                    Ok(CodeBlockType::Script(
                        ScriptName("example-script".to_string(),),
                        Some(ExitCode(2))
                    ))
                )
            }

            #[test]
            fn fails_when_name_is_missing() {
                let result = parse("shell,script()");
                assert_eq!(
                    result,
                    Err(Error::MissingArgument {
                        function: "script".to_string(),
                        argument: "name".to_string()
                    })
                )
            }
        }

        mod verify {
            use super::{parse, CodeBlockType, Error, ScriptName, Source, Stream};

            #[test]
            fn succeeds_when_function_is_verify_and_stream_is_stdout() {
                let result = parse(",verify(script_name=\"example-script\", stream=stdout)");
                assert_eq!(
                    result,
                    Ok(CodeBlockType::Verify(Source {
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
                    Ok(CodeBlockType::Verify(Source {
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
                        function: "verify".to_string(),
                        argument: "stream".to_string(),
                        expected: "output, stdout or stderr".to_string(),
                        got: "unknown".to_string(),
                    })
                )
            }

            #[test]
            fn fails_when_script_name_is_missing() {
                let result = parse("shell,verify(stream=stderr)");
                assert_eq!(
                    result,
                    Err(Error::MissingArgument {
                        function: "verify".to_string(),
                        argument: "script_name".to_string()
                    })
                )
            }

            #[test]
            fn fails_when_stream_is_missing() {
                let result = parse("shell,verify(script_name=\"the-script\")");
                assert_eq!(
                    result,
                    Err(Error::MissingArgument {
                        function: "verify".to_string(),
                        argument: "stream".to_string()
                    })
                )
            }
        }

        mod file {
            use super::{parse, CodeBlockType, Error, FilePath};

            #[test]
            fn succeeds_when_function_is_file() {
                let result = parse("text,file(path=\"example.txt\")");
                assert_eq!(
                    result,
                    Ok(CodeBlockType::CreateFile(FilePath(
                        "example.txt".to_string()
                    )))
                )
            }

            #[test]
            fn fails_when_path_is_missing() {
                let result = parse("text,file()");
                assert_eq!(
                    result,
                    Err(Error::MissingArgument {
                        function: "file".to_string(),
                        argument: "path".to_string()
                    })
                )
            }
        }
    }
}

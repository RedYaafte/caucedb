use crate::{Error, Result};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Statement {
    pub text: String,
    pub range: Range<usize>,
    pub line: usize,
}

// Byte offsets always point at ASCII delimiters or UTF-8 boundaries. A slash
// on its own line terminates PL/SQL, but never repeats the previous statement.
pub fn split(source: &str) -> Result<Vec<Statement>> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let (mut i, mut start) = (0, 0);
    let mut words: Vec<String> = Vec::new();
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if bytes[i..].starts_with(b"--") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i..].starts_with(b"/*") {
            i += 2;
            while i + 1 < bytes.len() && &bytes[i..i + 2] != b"*/" {
                i += 1;
            }
            if i + 1 >= bytes.len() {
                return Err(Error::Message("Unclosed SQL comment".into()));
            }
            i += 2;
            continue;
        }
        if (bytes[i] == b'q' || bytes[i] == b'Q') && bytes.get(i + 1) == Some(&b'\'') {
            let open = *bytes
                .get(i + 2)
                .ok_or_else(|| Error::Message("Unclosed q-quoted string".into()))?;
            let close = match open {
                b'[' => b']',
                b'{' => b'}',
                b'(' => b')',
                b'<' => b'>',
                c => c,
            };
            i += 3;
            while i + 1 < bytes.len() && !(bytes[i] == close && bytes[i + 1] == b'\'') {
                i += 1;
            }
            if i + 1 >= bytes.len() {
                return Err(Error::Message("Unclosed q-quoted string".into()));
            }
            i += 2;
            continue;
        }
        if bytes[i] == b'\'' || bytes[i] == b'"' {
            let quote = bytes[i];
            i += 1;
            let mut closed = false;
            while i < bytes.len() {
                if bytes[i] == quote {
                    i += 1;
                    if bytes.get(i) == Some(&quote) {
                        i += 1;
                    } else {
                        closed = true;
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            if !closed {
                return Err(Error::Message("Unclosed SQL string or identifier".into()));
            }
            continue;
        }
        let slash_line = bytes[i] == b'/' && {
            let a = source[..i].rfind('\n').map_or(0, |p| p + 1);
            let b = source[i..].find('\n').map_or(source.len(), |p| i + p);
            source[a..b].trim() == "/"
        };
        if slash_line || (bytes[i] == b';' && !procedural(&words)) {
            push(&mut out, source, start, i, !words.is_empty());
            i += 1;
            start = i;
            words.clear();
            continue;
        }
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            let a = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || b"_$#".contains(&bytes[i]))
            {
                i += 1;
            }
            if words.len() < 10 {
                words.push(source[a..i].to_ascii_uppercase());
            }
            continue;
        }
        i += 1;
    }
    push(&mut out, source, start, source.len(), !words.is_empty());
    for stmt in &out {
        let head = first_keyword(&stmt.text);
        if [
            "SET", "SPOOL", "PROMPT", "WHENEVER", "DEFINE", "UNDEFINE", "ACCEPT", "VARIABLE",
            "PRINT", "EXEC", "EXIT", "QUIT", "CONNECT", "HOST", "START",
        ]
        .contains(&head.as_str())
            || stmt.text.trim_start().starts_with('@')
        {
            return Err(Error::Message(format!(
                "SQL*Plus command at line {} is unsupported; use SQL or BEGIN ... END; /",
                stmt.line
            )));
        }
    }
    Ok(out)
}
fn push(out: &mut Vec<Statement>, source: &str, start: usize, end: usize, meaningful: bool) {
    if meaningful {
        let raw = &source[start..end];
        let begin = start + raw.len() - raw.trim_start().len();
        out.push(Statement {
            text: raw.trim().into(),
            range: start..end + usize::from(end < source.len()),
            line: 1 + source[..begin].bytes().filter(|b| *b == b'\n').count(),
        });
    }
}
fn procedural(words: &[String]) -> bool {
    if words
        .first()
        .is_some_and(|s| s == "BEGIN" || s == "DECLARE")
    {
        return true;
    }
    if words.first().is_some_and(|s| s == "CREATE") {
        let mut i = 1;
        if words.get(i).is_some_and(|s| s == "OR") {
            i += 2;
        }
        if words
            .get(i)
            .is_some_and(|s| s == "EDITIONABLE" || s == "NONEDITIONABLE")
        {
            i += 1;
        }
        return words.get(i).is_some_and(|s| {
            ["PROCEDURE", "FUNCTION", "TRIGGER", "PACKAGE", "TYPE"].contains(&s.as_str())
        });
    }
    false
}
pub fn first_keyword(mut sql: &str) -> String {
    loop {
        sql = sql.trim_start();
        if sql.starts_with("--") {
            sql = sql.split_once('\n').map_or("", |(_, s)| s);
        } else if sql.starts_with("/*") {
            sql = sql.split_once("*/").map_or("", |(_, s)| s);
        } else {
            break;
        }
    }
    sql.split(|c: char| !c.is_ascii_alphanumeric())
        .next()
        .unwrap_or("")
        .to_ascii_uppercase()
}
pub fn quote_identifier(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
pub fn literal(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sql_strings_comments_and_unicode() {
        let s = "-- comentario ;\nselect 'á;''b', q'[a;b]' from dual; /* ; */ select \"a;b\" from dual;";
        let v = split(s).unwrap();
        assert_eq!(v.len(), 2);
        assert!(v[0].text.contains("á;''b"));
    }
    #[test]
    fn procedural_blocks() {
        let s = "begin\n null;\n begin null; end;\nend;\n/\nselect 1 from dual;\ncreate or replace procedure p as begin null; end;\n/";
        let v = split(s).unwrap();
        assert_eq!(v.len(), 3);
        assert!(v[0].text.ends_with("end;"));
        assert!(v[2].text.ends_with("end;"));
    }
    #[test]
    fn rejects_incomplete_and_client_commands() {
        assert!(split("select 'x from dual;").is_err());
        assert!(split("/* unfinished").is_err());
        assert!(split("set serveroutput on\nselect 1 from dual;").is_err());
        assert!(split("-- only comment;").unwrap().is_empty());
    }
    #[test]
    fn slash_does_not_repeat() {
        assert_eq!(split("select 1 from dual;\n/\n").unwrap().len(), 1);
    }
    #[test]
    fn slash_inside_literal() {
        assert_eq!(split("begin x := q'[\n/\n;]'; end;\n/").unwrap().len(), 1);
    }
}

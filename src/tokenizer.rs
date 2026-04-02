#[derive(Debug, PartialEq, Eq)]
pub enum OperatorKind {
    Not,
    And,
    Or,
    Xor,
    Conditional,
    Biconditional,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Criteria {
    Name(String),
    Size(String),
    Type(String),
    Ctime(String),
    Atime(String),
    Mtime(String),
    Perm(String),
    Ext(String),
    Misc(String),
}

impl std::fmt::Display for Criteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(name) => write!(f, "{}", name),
            Self::Size(name) => write!(f, "{}", name),
            Self::Type(name) => write!(f, "{}", name),
            Self::Ctime(name) => write!(f, "{}", name),
            Self::Atime(name) => write!(f, "{}", name),
            Self::Mtime(name) => write!(f, "{}", name),
            Self::Perm(name) => write!(f, "{}", name),
            Self::Ext(name) => write!(f, "{}", name),
            Self::Misc(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Tokens {
    Operator(OperatorKind),
    Operand(Criteria)
}

impl std::fmt::Display for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Operand(op) => write!(f, "{}", op),
            Self::Operator(op) => write!(f, "{:?}", op),
        }
    }
}


pub fn tokenize(predicate: &str) -> Result<Vec<Tokens>, String> {
    let mut tokens = Vec::new();
    let mut parts = predicate.split(' ');

    while let Some(token) = parts.next() {
        let next_token = match token {
            "!"  => Tokens::Operator(OperatorKind::Not),
            "&"  => Tokens::Operator(OperatorKind::And),
            "|"  => Tokens::Operator(OperatorKind::Or),
            "^"  => Tokens::Operator(OperatorKind::Xor),
            "="  => Tokens::Operator(OperatorKind::Biconditional),
            "~"  => Tokens::Operator(OperatorKind::Conditional),
            _ if token.starts_with("type:")  => Tokens::Operand(Criteria::Type(token.strip_prefix("type:").unwrap().to_string())),
            _ if token.starts_with("perm:")   => Tokens::Operand(Criteria::Perm(token.strip_prefix("perm:").unwrap().to_string())),
            _ if token.starts_with("mtime:")  => Tokens::Operand(Criteria::Mtime(token.strip_prefix("mtime:").unwrap().to_string())),
            _ if token.starts_with("atime:")  => Tokens::Operand(Criteria::Atime(token.strip_prefix("atime:").unwrap().to_string())),
            _ if token.starts_with("ctime:")  => Tokens::Operand(Criteria::Ctime(token.strip_prefix("ctime:").unwrap().to_string())),
            _ if token.starts_with("misc:")  => Tokens::Operand(Criteria::Misc(token.strip_prefix("misc:").unwrap().to_string())),
            _ if token.starts_with("ext:")  => Tokens::Operand(Criteria::Ext(token.strip_prefix("ext:").unwrap().to_string())),
            _ if token.starts_with("size:")  => Tokens::Operand(Criteria::Size(token.strip_prefix("size:").unwrap().to_string())),
            _ if token.starts_with("name:")  => Tokens::Operand(
                Criteria::Name(collect_whole_name(&mut parts, token.strip_prefix("name:").unwrap())?)),
            _ => return Err(format!("Unexpected token: {}", token))
        };
        tokens.push(next_token);
    }
    Ok(tokens)
}

fn collect_whole_name<'a, S: Iterator<Item = &'a str>>(source: &mut S, first_token: &str) -> Result<String, &'static str> {
    if !first_token.starts_with("\"") {
        return Ok(first_token.to_string())
    }
    let stripped_first_token = first_token.strip_prefix("\"").unwrap();
    if contains_valid_closing_quote(stripped_first_token)? {
        return Ok(stripped_first_token.strip_suffix("\"").unwrap().to_string())
    }
    let mut name_sink = String::new();
    name_sink.push_str(stripped_first_token);
    while let Some(token) = source.next() {
        if contains_valid_closing_quote(token)? {
            name_sink.push_str(" ");
            name_sink.push_str(token.strip_suffix("\"").unwrap());
            return Ok(name_sink.replace("\\\"", "\""))
        }
        name_sink.push_str(" ");
        name_sink.push_str(token);
    };
    Err("unclosed quote")
}

pub fn contains_valid_closing_quote(token: &str) -> Result<bool, &'static str> {
    if token.is_empty() {
        return Ok(false);
    }
    let mut chars = token.chars().peekable();
    while let Some(char) = chars.next() {
        match char {
            '\\' => {chars.next();},
            '"' => {
                match &chars.peek() {
                    None => return Ok(true),
                    _ => return Err("unescaped quote at unexpected place"),
                }
            },
            _ => continue
        }
    };
    Ok(false)
}
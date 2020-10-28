#![allow(dead_code)]

#[derive(Debug, Eq, PartialEq)]
pub enum Form {
    Str(String),
    Sym(String),
    Keyword(String),
    List(Vec<Form>),
}

pub struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize
}

impl<'a> Parser<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
	Self {
	    bytes,
	    pos: 0
	}
    }

    fn peek(&self) -> Option<u8> {
	self.bytes.get(self.pos).cloned()
    }

    fn bump(&mut self) {
	self.pos += 1;
    }

    fn is_ws(&self) -> bool {
	match self.peek() {
	    Some(b' ') | Some(b'\n') | Some(b',') => true,
	    _ => false
	}
    }

    fn parse_string(&mut self) -> Option<Form> {
	self.bump();
	let mut chars = vec![];
	
	let mut c = self.peek()?;
	while c != b'"' {
	    chars.push(c);
	    self.bump();
	    if let Some(nc) = self.peek() {
		c = nc;
	    } else {
		break;
	    } 
	}
	self.bump();

	Some(Form::Str(unsafe { String::from_utf8_unchecked(chars) }))
    }

    fn parse_sym(&mut self) -> Option<Form> {
	let mut chars = vec![];
	let mut c = self.peek()?;
	while !self.is_ws() && c != b')' {
	    chars.push(c);
	    self.bump();
	    if let Some(nc) = self.peek() {
		c = nc;
	    } else {
		break;
	    }
	}

	Some(Form::Sym(unsafe { String::from_utf8_unchecked(chars) }))
    }

    fn parse_form(&mut self) -> Option<Form> {
	while self.is_ws() {
	    self.bump();
	}
	
	match self.peek()? {
	    b'"' => self.parse_string(),
	    b'(' => self.parse_list(),
	    b':' => {
		self.bump();
		match self.parse_sym() {
		    Some(Form::Sym(s)) => Some(Form::Keyword(s)),
		    _ => None
		}
	    }
	    c => self.parse_sym(),
	}
    }

    fn parse_list(&mut self) -> Option<Form> {
	self.bump();
	let mut forms = vec![];

	while self.peek() != Some(b')')  {
	    if let Some(form) = self.parse_form() {
		forms.push(form);
	    } else {
		break;
	    }
	}
	self.bump();

	Some(Form::List(forms))
    }
}


#[cfg(test)]
mod test {
    use super::*;

    macro_rules! sym {
	($e:expr) => {
	    Form::Sym($e.to_string())
	}
    }

    macro_rules! kw {
	($e:expr) => {
	    Form::Keyword($e.to_string())
	}
    }

    macro_rules! string {
	($e:expr) => {
	    Form::Str($e.to_string())
	}
    }

    macro_rules! list {
	($($e:expr),*) => {
	    Form::List(vec![
		$($e),*
	    ])
	}
    }

    #[test]
    fn parses_strings() {
	let input = r#""the quick brown fox""#;
	let mut parser = Parser::from_bytes(&input.as_bytes());
	assert_eq!(parser.parse_form(), Some(string!("the quick brown fox")))
    }

    #[test]
    fn parses_syms() {
	let input = r#"the "#;
	let mut parser = Parser::from_bytes(&input.as_bytes());
	assert_eq!(parser.parse_form(), Some(sym!("the")))
    }

    #[test]
    fn parses_keywords() {
	let input = r#":the"#;
	let mut parser = Parser::from_bytes(&input.as_bytes());
	assert_eq!(parser.parse_form(), Some(kw!("the")))
    }

    #[test]
    fn parses_lists() {
	let input = r#"(the quick brown "fox")"#;
	let mut parser = Parser::from_bytes(&input.as_bytes());
	assert_eq!(parser.parse_form(), Some(list!(sym!("the"), sym!("quick"), sym!("brown"), string!("fox"))))
    }
}

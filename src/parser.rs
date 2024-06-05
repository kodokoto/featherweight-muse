use crate::ast::{Argument, LVal, Program, Term, Value};
use crate::token::Token;
use crate::typing::Type;

pub struct Parser {
    tokens: Vec<Token>,
    current_position: usize,
}

impl Parser {
    fn check_consume(&mut self, token: Token) {
        match self.tokens.get(self.current_position) {
            Some(t) => {
                if t == &token {
                    self.current_position += 1;
                } else {
                    panic!("Expected {:?}, got {:?}", token, t);
                }
            }
            None => panic!("Unexpected EOF"),
        }
    }

    fn parse_let(&mut self) -> Term {
        self.check_consume(Token::Let);
        self.check_consume(Token::Mut);
        let variable = self.parse_variable();
        self.check_consume(Token::Assign);
        let term = self.parse_term();
        Term::Let {
            mutable: true,
            variable,
            term: Box::new(term),
        }
    }

    fn parse_assignment(&mut self) -> Term {
        let variable = self.parse_variable();
        self.check_consume(Token::Assign);
        let term = self.parse_term();
        println!("Parsed assignment: {:?} = {:?}", variable, term);
        Term::Assign {
            variable,
            term: Box::new(term),
        }
    }

    fn parse_function_declaration(&mut self) -> Term {
        self.check_consume(Token::Fn);
        let name: String = match self.tokens.get(self.current_position) {
            Some(Token::Identifier(s)) => {
                self.current_position += 1;
                s.to_string()
            }
            _ => panic!("Expected identifier"),
        };

        self.check_consume(Token::LParen);
        let args = self.parse_args();
        self.check_consume(Token::RParen);

        let ty = match self.tokens.get(self.current_position) {
            Some(Token::Colon) => {
                self.current_position += 1;
                Some(self.parse_type())
            }
            _ => None,
        };

        self.check_consume(Token::LCurl);

        let mut body = Vec::new();
        loop {
            let term = self.parse_term();
            body.push(term);
            if self.tokens.get(self.current_position) == Some(&Token::RCurl) {
                break;
            }
            if self.tokens.get(self.current_position) == None {
                break;
            }
        }

        self.check_consume(Token::RCurl);

        Term::FunctionDeclaration {
            name,
            args,
            body,
            ty,
        }
    }

    fn parse_type(&mut self) -> Type {
        match self.tokens.get(self.current_position) {
            Some(Token::Identifier(s)) => {
                self.current_position += 1;
                match s.as_str() {
                    "int" => Type::Numeric,
                    _ => panic!("Expected int type"),
                }
            }
            Some(Token::Box) => {
                self.current_position += 1;
                Type::Box(Box::new(self.parse_type()))
            }
            _ => panic!("Expected type"),
        }
    }

    fn parse_args(&mut self) -> Vec<Argument> {
        let mut args = Vec::new();
        loop {
            match self.tokens.get(self.current_position) {
                Some(Token::RParen) => {
                    break;
                }
                Some(Token::Comma) => {
                    self.current_position += 1;
                }
                Some(Token::Identifier(s)) => {
                    self.current_position += 1;
                    let name = s.to_string();
                    self.check_consume(Token::Colon);
                    let ty = self.parse_type();
                    args.push(Argument {
                        name,
                        mutable: false,
                        reference: false,
                        ty,
                    });
                }
                Some(Token::Ref) => {
                    self.current_position += 1;
                    let name = match self.tokens.get(self.current_position) {
                        Some(Token::Identifier(s)) => {
                            self.current_position += 1;
                            s.to_string()
                        }
                        _ => panic!("Expected identifier"),
                    };
                    self.check_consume(Token::Colon);
                    let ty = self.parse_type();
                    args.push(Argument {
                        name,
                        mutable: false,
                        reference: true,
                        ty,
                    });
                }
                Some(Token::Mut) => {
                    self.current_position += 1;
                    let reference = match self.tokens.get(self.current_position) {
                        Some(Token::Ref) => {
                            self.current_position += 1;
                            true
                        }
                        _ => false,
                    };
                    let name = match self.tokens.get(self.current_position) {
                        Some(Token::Identifier(s)) => {
                            self.current_position += 1;
                            s.to_string()
                        }
                        _ => panic!("Expected identifier"),
                    };
                    self.check_consume(Token::Colon);
                    let ty = self.parse_type();
                    args.push(Argument {
                        name,
                        mutable: true,
                        reference,
                        ty,
                    });
                }
                _ => panic!("Expected identifier or comma"),
            }
        }
        args
    }

    fn parse_function_call(&mut self) -> Term {
        let name = match self.tokens.get(self.current_position) {
            Some(Token::Identifier(s)) => {
                self.current_position += 1;
                s.to_string()
            }
            _ => panic!("Expected identifier"),
        };
        self.check_consume(Token::LParen);

        let mut params = Vec::new();
        loop {
            match self.tokens.get(self.current_position) {
                Some(Token::RParen) => {
                    break;
                }
                Some(Token::Comma) => {
                    self.current_position += 1;
                }
                _ => {
                    let term = self.parse_term();
                    params.push(term);
                }
            }
        }
        self.check_consume(Token::RParen);
        Term::FunctionCall { name, params }
    }

    fn parse_term(&mut self) -> Term {
        match self.tokens.get(self.current_position) {
            Some(token) => {
                match token {
                    Token::Fn => self.parse_function_declaration(),
                    Token::NumericLiteral(n) => {
                        self.current_position += 1;
                        Term::Value(Value::NumericLiteral(*n))
                    }
                    Token::Identifier(s) => {
                        // check if assignment
                        if self.tokens.get(self.current_position + 1) == Some(&Token::Assign) {
                            self.parse_assignment()
                        } else if self.tokens.get(self.current_position + 1) == Some(&Token::LParen)
                        {
                            self.parse_function_call()
                        } else {
                            self.current_position += 1;
                            Term::Variable(LVal::Variable {
                                name: s.to_string(),
                                copyable: None,
                            })
                        }
                    }
                    Token::Box => {
                        self.current_position += 1;
                        Term::Box {
                            term: Box::new(self.parse_term()),
                        }
                    }
                    Token::Mut => {
                        // should only be mut ref
                        self.current_position += 1;
                        self.check_consume(Token::Ref);
                        // current token:
                        println!("{:?}", self.tokens.get(self.current_position));
                        Term::Ref {
                            mutable: true,
                            var: self.parse_variable(),
                        }
                    }
                    Token::Ref => {
                        self.current_position += 1;
                        Term::Ref {
                            mutable: false,
                            var: self.parse_variable(),
                        }
                    }
                    Token::Let => self.parse_let(),

                    Token::Deref => {
                        if self.tokens.get(self.current_position + 2) == Some(&Token::Assign) {
                            self.parse_assignment()
                        } else {
                            self.current_position += 1;
                            Term::Variable(LVal::Deref {
                                var: Box::new(self.parse_variable()),
                            })
                        }
                    }
                    _ => panic!("Invalid token: {:?}", token),
                }
            }
            None => panic!("Unexpected EOF"),
        }
    }

    fn parse_variable(&mut self) -> LVal {
        match self.tokens.get(self.current_position) {
            Some(token) => match token {
                Token::Identifier(s) => {
                    self.current_position += 1;
                    LVal::Variable {
                        name: s.to_string(),
                        copyable: None,
                    }
                }
                Token::Deref => {
                    self.current_position += 1;
                    let var = self.parse_variable();
                    LVal::Deref { var: Box::new(var) }
                }
                _ => panic!("Invalid token: {:?}, expected variable", token),
            },
            None => panic!("Unexpected EOF"),
        }
    }

    pub fn parse(&mut self) -> Program {
        let mut terms = Vec::new();
        loop {
            let term = self.parse_term();
            terms.push(term);
            if self.tokens.get(self.current_position) == None {
                break;
            }
        }
        Program { terms }
    }

    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current_position: 0,
        }
    }
}


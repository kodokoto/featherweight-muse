
use crate::token::TokenKind;
use crate::ast::{Argument, Declaration, Path, Program, Term, Value, LVal};
use crate::typing::Type;

pub struct Parser
{
    tokens: Vec<TokenKind>,
    current_position: usize,
}

impl Parser {

    fn check_consume(&mut self, token: TokenKind)  {
        match self.tokens.get(self.current_position) {
            Some(t) => {
                if t == &token {
                    self.current_position += 1;
                } else {
                    panic!("Expected {:?}, got {:?}", token, t);
                }
            },
            None => panic!("Unexpected EOF")
        }
    }

    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.current_position)
    }

    fn check_if_mut(&mut self) -> bool {
        match self.peek() {
            Some(token) => {
                match token {
                    TokenKind::Mut => {
                        self.current_position += 1;
                        true
                    },
                    _ => false
                }
            },
            None => false
        }
    }

    fn parse_let(&mut self) -> Term {
        self.check_consume(TokenKind::Let);
        // let mutable = self.check_if_mut(); all variables are mut for now
        self.check_consume(TokenKind::Mut);
        let variable = self.parse_variable(false);
        self.check_consume(TokenKind::Assign);
        let term = self.parse_term();
        Term::Let {
            mutable: true,
            variable,
            term: Box::new(term)
        }
    }

    fn parse_assignment(&mut self) -> Term {
        let variable = self.parse_variable(true);
        self.check_consume(TokenKind::Assign);
        let term = self.parse_term();
        println!("Parsed assignment: {:?} = {:?}", variable, term);
        Term::Assign {
            variable,
            term: Box::new(term)
        }
    }

    fn parse_function_declaration(&mut self) -> Term {
        self.check_consume(TokenKind::Fn);
        // expect identifier

        let name: String = match self.tokens.get(self.current_position) {
            Some(TokenKind::Identifier(s)) => {
                self.current_position += 1;
                s.to_string()
            },
            _ => panic!("Expected identifier")
        };

        self.check_consume(TokenKind::LParen);
        let args = self.parse_args();
        self.check_consume(TokenKind::RParen);

        let ty = match self.tokens.get(self.current_position) {
            Some(TokenKind::Colon) => {
                self.current_position += 1;
                Some(self.parse_type())
            },
            _ => None
        };

        self.check_consume(TokenKind::LCurl);

        let mut body = Vec::new();
        loop {
            let term = self.parse_term();
            body.push(term);
            if self.tokens.get(self.current_position) == Some(&TokenKind::RCurl) {
                break;
            }
            if self.tokens.get(self.current_position) == None {
                break;
            }
        }

        self.check_consume(TokenKind::RCurl);

        Term::FunctionDeclaration {
            name,
            args,
            body,
            ty
        }
    }

    fn parse_type(&mut self) -> Type {
        match self.tokens.get(self.current_position) {
            Some(TokenKind::Identifier(s)) => {
                self.current_position += 1;
                match s.as_str() {
                    "int" => Type::Numeric,
                    _ => panic!("Expected int type")
                }
            },
            Some(TokenKind::Box) => {
                self.current_position += 1;
                Type::Box(Box::new(self.parse_type()))
            },
            _ => panic!("Expected type")
        }
    }

    fn parse_args(&mut self) -> Vec<Argument> {
        let mut args = Vec::new();
        loop {
            match self.tokens.get(self.current_position) {
                Some(TokenKind::RParen) => {
                    break;
                },
                Some(TokenKind::Comma) => {
                    self.current_position += 1;
                },
                Some(TokenKind::Identifier(s)) => {
                    self.current_position += 1;
                    let name = s.to_string();
                    self.check_consume(TokenKind::Colon);
                    let ty = self.parse_type();
                    args.push(Argument {
                        name,
                        mutable: false,
                        reference: false,
                        ty
                    });
                },
                Some(TokenKind::Ref) => {
                    self.current_position += 1;
                    let name = match self.tokens.get(self.current_position) {
                        Some(TokenKind::Identifier(s)) => {
                            self.current_position += 1;
                            s.to_string()
                        },
                        _ => panic!("Expected identifier")
                    };
                    self.check_consume(TokenKind::Colon);
                    let ty = self.parse_type();
                    args.push(Argument {
                        name,
                        mutable: false,
                        reference: true,
                        ty
                    });
                },
                Some(TokenKind::Mut) => {
                    self.current_position += 1;
                    let reference = match self.tokens.get(self.current_position) {
                        Some(TokenKind::Ref) => {
                            self.current_position += 1;
                            true
                        },
                        _ => false
                    };
                    let name = match self.tokens.get(self.current_position) {
                        Some(TokenKind::Identifier(s)) => {
                            self.current_position += 1;
                            s.to_string()
                        },
                        _ => panic!("Expected identifier")
                    };
                    self.check_consume(TokenKind::Colon);
                    let ty = self.parse_type();
                    args.push(Argument {
                        name,
                        mutable: true,
                        reference,
                        ty
                    });
                },
                _ => panic!("Expected identifier or comma")
            }
        }
        args
    }

    fn parse_function_call(&mut self) -> Term {
        let name = match self.tokens.get(self.current_position) {
            Some(TokenKind::Identifier(s)) => {
                self.current_position += 1;
                s.to_string()
            },
            _ => panic!("Expected identifier")
        };
        self.check_consume(TokenKind::LParen);
    
        let mut params = Vec::new();
        loop {
            match self.tokens.get(self.current_position) {
                Some(TokenKind::RParen) => {
                    break;
                },
                Some(TokenKind::Comma) => {
                    self.current_position += 1;
                },
                _ => {
                    let term = self.parse_term();
                    params.push(term);
                }
            }
        }
        self.check_consume(TokenKind::RParen);
        Term::FunctionCall {
            name,
            params
        }
    }

    fn parse_term(&mut self) -> Term {
        match self.tokens.get(self.current_position) {
            Some(token) => {
                match token {
                    TokenKind::Fn => {
                        self.parse_function_declaration()
                    },
                    TokenKind::NumericLiteral(n) => {
                        self.current_position += 1;
                        Term::Value(Value::NumericLiteral(*n))
                    },
                    TokenKind::Identifier(s) => {
                        // check if assignment
                        if self.tokens.get(self.current_position + 1) == Some(&TokenKind::Assign) {
                            self.parse_assignment()
                        } else if self.tokens.get(self.current_position + 1) == Some(&TokenKind::LParen) {
                            self.parse_function_call()
                        } else {
                            self.current_position += 1;
                            Term::Variable(
                                LVal::Variable {
                                    name: s.to_string(),
                                    copyable: None
                                }
                            )
                        }
                    },
                    // Token::Move => {
                    //     self.current_position += 1;
                    //     Term::Move {
                    //         variable: self.parse_variable()
                    //     }
                    // },
                    // Token::Copy => {
                    //     self.current_position += 1;
                    //     Term::Copy {
                    //         variable: self.parse_variable()
                    //     }
                    // },
                    TokenKind::Box => {
                        self.current_position += 1;
                        Term::Box {
                            term: Box::new(self.parse_term())
                        }
                    },
                    TokenKind::Mut => {
                        // should only be mut ref
                        self.current_position += 1;
                        self.check_consume(TokenKind::Ref);
                        // current token:
                        println!("{:?}", self.tokens.get(self.current_position));
                        Term::Ref {
                            mutable: true,
                            var: self.parse_variable(false)
                        }
                    },
                    TokenKind::Ref => {
                        self.current_position += 1;
                        Term::Ref {
                            mutable: false,
                            var: self.parse_variable(false)
                        }
                    },
                    TokenKind::Let => {
                        self.parse_let()
                    },

                    TokenKind::Deref => {
                        if self.tokens.get(self.current_position + 2) == Some(&TokenKind::Assign) {
                            self.parse_assignment()
                        } else {
                        self.current_position += 1;
                        Term::Variable(
                            LVal::Deref {
                                var: Box::new(self.parse_variable(false))
                            }
                        )}
                    },
                    _ => panic!("Invalid token: {:?}", token)
                }
            },
            None => panic!("Unexpected EOF")
        }
    }

    fn parse_variable(&mut self, deref: bool) -> LVal {
        match self.tokens.get(self.current_position) {
            Some(token) => {
                match token {
                    TokenKind::Identifier(s) => {
                        self.current_position += 1;
                        LVal::Variable {
                            name: s.to_string(),
                            copyable: None
                        }
                    },
                    TokenKind::Deref => {
                        self.current_position += 1;
                        let var = self.parse_variable(true);
                        LVal::Deref {
                            var: Box::new(var)
                        }
                    }
                    _ => panic!("Invalid token: {:?}, expected variable", token)
                }
            },
            None => panic!("Unexpected EOF")
        }
    }

    pub fn parse(&mut self) -> Program {
        let mut terms = Vec::new();
        // let mut declarations = Vec::new();
        loop {
            // if (self.tokens.get(self.current_position) == Some(&Token::Fn)) {
            //     let declaration = self.parse_function_declaration();
            //     declarations.push(declaration);
            // }
            let term = self.parse_term();
            terms.push(term);
            if self.tokens.get(self.current_position) == None {
                break;
            }
        }
        Program {
            terms,
            // declarations
        }
    }

    pub fn new(tokens: Vec<TokenKind>) -> Parser {
        Parser {
            tokens,
            current_position: 0
        }
    }
}
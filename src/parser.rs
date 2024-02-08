use crate::token::Token;
use crate::ast::{Term, Value, Variable, Program, AST};

pub struct Parser
{
    tokens: Vec<Token>,
    current_position: usize,
}

impl Parser {

    fn check_consume(&mut self, token: Token)  {
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

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current_position)
    }

    fn check_if_mut(&mut self) -> bool {
        match self.peek() {
            Some(token) => {
                match token {
                    Token::Mut => {
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
        self.check_consume(Token::Let);
        let mutable = self.check_if_mut();
        let variable = self.parse_variable();
        self.check_consume(Token::Assign);
        let term = self.parse_term();
        Term::Let {
            mutable,
            variable,
            term: Box::new(term)
        }
    }

    fn parse_assignment(&mut self) -> Term {
        let variable = self.parse_variable();
        self.check_consume(Token::Assign);
        let term = self.parse_term();
        Term::Assign {
            variable,
            term: Box::new(term)
        }
    }

    fn parse_term(&mut self) -> Term {
        match self.tokens.get(self.current_position) {
            Some(token) => {
                match token {
                    Token::NumericLiteral(n) => {
                        self.current_position += 1;
                        Term::Value(Value::NumericLiteral(*n))
                    },
                    Token::Identifier(s) => {
                        // check if assignment
                        if self.tokens.get(self.current_position + 1) == Some(&Token::Assign) {
                            self.parse_assignment()
                        } else {
                            self.current_position += 1;
                            Term::Variable(Variable {
                                name: s.to_string()
                            })
                        }
                    },
                    Token::Move => {
                        self.current_position += 1;
                        Term::Move {
                            variable: self.parse_variable()
                        }
                    },
                    Token::Copy => {
                        self.current_position += 1;
                        Term::Copy {
                            variable: self.parse_variable()
                        }
                    },
                    Token::Box => {
                        self.current_position += 1;
                        Term::Box {
                            term: Box::new(self.parse_term())
                        }
                    },
                    Token::Mut => {
                        // should only be mut ref
                        self.current_position += 1;
                        self.check_consume(Token::Ref);
                        Term::Ref {
                            mutable: true,
                            term: Box::new(self.parse_term())
                        }
                    },
                    Token::Ref => {
                        self.current_position += 1;
                        Term::Ref {
                            mutable: false,
                            term: Box::new(self.parse_term())
                        }
                    },
                    Token::Let => {
                        self.parse_let()
                    },
                    _ => panic!("Invalid token: {:?}", token)
                }
            },
            None => panic!("Unexpected EOF")
        }
    }

    fn parse_variable(&mut self) -> Variable {
        match self.tokens.get(self.current_position) {
            Some(token) => {
                match token {
                    Token::Identifier(s) => {
                        self.current_position += 1;
                        Variable {
                            name: s.to_string()
                        }
                    },
                    _ => panic!("Invalid token: {:?}, expected variable", token)
                }
            },
            None => panic!("Unexpected EOF")
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
        Program {
            terms
        }
    }

    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current_position: 0
        }
    }
}
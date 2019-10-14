use std::io::BufRead;

macro_rules! read_line(
    () => {{
        let stdin = std::io::stdin();
        let stdin = stdin.lock();
        let mut lines = stdin.lines();
        lines.next().unwrap()
    }}
);


enum TokenIndexes {
    Number(usize, usize),
    ProdOp(usize, usize),
    TermOp(usize, usize),
}


#[derive(Debug)]
enum Token {
    Number(f64),
    ProdOp(char),
    TermOp(char),
    None,
}


struct CalcMem {
    main_stack: Vec<Token>,
    aux_stack: Vec<Token>,
    op_stack: Vec<Token>,
    current: Token,
}

/// Takes a valid string and returns a vector holding token types and their
/// indexes in the string in a TokenIndexes enum.
fn index_tokens(input: &String) -> Result<Vec<TokenIndexes>, &str> {
    let mut indexes: Vec<TokenIndexes> = Vec::new();

    let mut in_number: bool = false;
    for (i, c) in input.chars().enumerate() {
        match c {
            '0'..='9' | '.' => {
                if in_number {
                    if let TokenIndexes::Number(a, b) = indexes.pop().unwrap() {
                        indexes.push(TokenIndexes::Number(a, i+1));
                    }
                } else {
                    indexes.push(TokenIndexes::Number(i, i+1));
                }
                in_number = true;
            },
            '*' | '/' => {
                indexes.push(TokenIndexes::ProdOp(i, i+1));
                in_number = false;
            },
            '+' | '-' => {
                indexes.push(TokenIndexes::TermOp(i, i+1));
                in_number = false;
            },
            ' ' => {
                in_number = false;
            },
            ',' | '\'' => { return Err("Do not use delimiters such as , or '.") },
            _ => { return Err("Invalid input! Only numbers, *, /, +, -, and spaces allowed.") }
        }
    }
    Ok(indexes)
}

/// Takes a Token vector in infix form and returns a Token vector in postfix form
fn to_postfix(tokens: &mut Vec<Token>) -> Vec<Token> {

    let mut calc_mem = CalcMem {
        main_stack: Vec::new(),
        aux_stack: Vec::new(),
        op_stack: Vec::new(),
        current: Token::None,
    };


    let mut waiting_number = false;
    let mut waiting_operator = false;

    loop {
        calc_mem.current = match tokens.pop() {
            Some(t) => t,
            None => Token::None,
        };

        match calc_mem.current {
            n @ Token::Number(_) => {
                if waiting_number {
                    waiting_number = false;
                    calc_mem.aux_stack.push(n);
                    let op = calc_mem.op_stack.pop().unwrap();
                    calc_mem.aux_stack.push(op);
                }
                else {
                    calc_mem.aux_stack.push(n);
                }
            },

            p @ Token::ProdOp(_) => {
                calc_mem.op_stack.push(p);
                waiting_number = true;
            },

            t @ Token::TermOp(_) => {
                loop {
                    let token = match calc_mem.aux_stack.pop() {
                        Some(t) => t,
                        None => { break; }
                    };
                    calc_mem.op_stack.push(token);
                }
                loop {
                    let token = match calc_mem.op_stack.pop() {
                        Some(t) => t,
                        None => { break; }
                    };
                    calc_mem.main_stack.push(token);
                }
                calc_mem.op_stack.push(t);
            },

            Token::None => { 
                loop {
                    let token = match calc_mem.aux_stack.pop() {
                        Some(t) => t,
                        None => { break; }
                    };
                    calc_mem.op_stack.push(token);
                }
                loop {
                    let token = match calc_mem.op_stack.pop() {
                        Some(t) => t,
                        None => { break; }
                    };
                    calc_mem.main_stack.push(token);
                }
                break;
            },
        };
    };
    calc_mem.main_stack
}

/// Takes a string and returns a Result holding a Token vector in postfix form
fn tokenize(input: &String) -> Result<Vec<Token>, &str> {

    let mut indexes = match index_tokens(&input) {
        Ok(vector) => vector,
        Err(error) => { return Err(error); }
    };

    let mut tokens = Vec::new();

    for index_pair in indexes.into_iter() {
        tokens.push( match index_pair {
            TokenIndexes::Number(a, b) => Token::Number(input[a..b].parse().unwrap()),
            TokenIndexes::ProdOp(a, b) => Token::ProdOp(input[a..b].parse().unwrap()),
            TokenIndexes::TermOp(a, b) => Token::TermOp(input[a..b].parse().unwrap()),
        });
    }

    Ok(tokens)
}


fn main() {
    let input: String = read_line!().expect("Error reading line");
    let mut tokens = match tokenize(&input) {
        Ok(a) => a,
        Err(b) => {
            eprintln!("{}", b);
            return
        },
    };
    println!("{:?}", tokens);
    let tokens = to_postfix(&mut tokens);
    println!("{:?}", tokens);
}

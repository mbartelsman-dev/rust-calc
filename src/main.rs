use std::io::BufRead;
#[macro_use] extern crate assert_matches; // For unit tests


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


#[derive(Debug, Clone)]
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

            // If currently parsing a number, continue. Else, begin new num.
            '0'..='9' | '.' => {
                if in_number {
                    if let TokenIndexes::Number(a, _) = indexes.pop().unwrap() {
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


/// Takes a string and returns a Result holding a Token vector in postfix form
fn tokenize(input: &String) -> Result<Vec<Token>, &str> {

    let indexes = match index_tokens(&input) {
        Ok(vector) => vector,
        Err(error) => { return Err(error); }
    };

    let mut tokens = Vec::new();

    // Convert indices to tokens
    for index_pair in indexes.into_iter() {
        tokens.push( match index_pair {
            TokenIndexes::Number(a, b) => Token::Number(input[a..b].parse().unwrap()),
            TokenIndexes::ProdOp(a, b) => Token::ProdOp(input[a..b].parse().unwrap()),
            TokenIndexes::TermOp(a, b) => Token::TermOp(input[a..b].parse().unwrap()),
        });
    }

    Ok(tokens)
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

    // Algorithm works as follows:
    // Read expression token by token
    loop {
        calc_mem.current = match tokens.pop() {
            Some(t) => t,
            None => Token::None,
        };

        match calc_mem.current {

            // If token is a number
            // Then push number to aux stack,
            // If an operand was waiting, push the operand to the aux stack afterwards
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

            // If the token is a mut/div operand, push to operand stack and wait for number
            p @ Token::ProdOp(_) => {
                calc_mem.op_stack.push(p);
                waiting_number = true;
            },

            // If token is a plus/min operand, pop and push entre aux stack onto operand stack, reversing it
            // Then do the same from the operand stack to the main stack
            // Finally, push current operand (+/-) to operand stack
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

            // If expression list is empty, empty stacks onto main stack
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

/// Recursively evaluates a vector holding tokens in postfix form, returns a Result
fn eval_postfix<'a>(mut expr: &mut Vec<Token>) -> Result<f64, &'a str> {

    // If expression is empty, return error
    // If next item is a number, return it
    let op = match expr.pop() {
        Some(e) => match e {
            Token::None => { return Err("Expected something, found nothing"); },
            Token::Number(num) => { return Ok(num); },
            w => w
        }
        None => { return Err("Expected something, found nothing"); }
    };

    // Eval first sub-expression recursively
    let a = match eval_postfix(&mut expr) {
        Ok(n) => n,
        Err(e) => { return Err(e); }
    };

    // Eval second sub-expression recursively 
    let b = match eval_postfix(&mut expr) {
        Ok(n) => n,
        Err(e) => { return Err(e); }
    };

    // Perform appropriate operation
    let ans = match op {
        Token::TermOp('+') => a + b,
        Token::TermOp('-') => a - b,
        Token::TermOp(_) => { return Err("Invalid Token"); }
        Token::ProdOp('*') => a * b,
        Token::ProdOp('/') => {
            if b == 0.0 { return Err("Undefined: Division by zero"); }
            a / b
        },
        Token::ProdOp(_) => { return Err("Invalid Token"); }
        _ => 0.0  // Compiler complained without this even though it's covered in the base case
    };

    Ok(ans)
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
    let mut tokens = to_postfix(&mut tokens);
    println!("{:?}", tokens);
    let answer = eval_postfix(&mut tokens).unwrap();
    println!("Answer is {}", answer);
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn eval_postfix_test() {

        // 1 + 1 = 2
        let mut expr = vec![Token::Number(1.), Token::Number(1.), Token::TermOp('+')];
        assert_eq!(Ok(2.0),eval_postfix(&mut expr));

        // 1 - 1 = 0
        let mut expr = vec![Token::Number(1.), Token::Number(1.), Token::TermOp('-')];
        assert_eq!(Ok(0.0),eval_postfix(&mut expr));

        // 1 * 1 = 1
        let mut expr = vec![Token::Number(1.), Token::Number(1.), Token::ProdOp('*')];
        assert_eq!(Ok(1.0),eval_postfix(&mut expr));

        // 2 * 1 = 2
        let mut expr = vec![Token::Number(2.), Token::Number(1.), Token::ProdOp('*')];
        assert_eq!(Ok(2.0),eval_postfix(&mut expr));

        // 2 * 0 = 0
        let mut expr = vec![Token::Number(2.), Token::Number(0.), Token::ProdOp('*')];
        assert_eq!(Ok(0.0),eval_postfix(&mut expr));

        // 1 / 1 = 1
        let mut expr = vec![Token::Number(1.), Token::Number(1.), Token::ProdOp('/')];
        assert_eq!(Ok(1.0),eval_postfix(&mut expr));

        // 2 / 2 = 1
        let mut expr = vec![Token::Number(2.), Token::Number(2.), Token::ProdOp('/')];
        assert_eq!(Ok(1.0),eval_postfix(&mut expr));

        // 2 / 1 = 2
        let mut expr = vec![Token::Number(1.), Token::Number(2.), Token::ProdOp('/')];
        assert_eq!(Ok(2.0),eval_postfix(&mut expr));

        // 2 / 0 = Err
        let mut expr = vec![Token::Number(0.), Token::Number(2.), Token::ProdOp('/')];
        let ans = eval_postfix(&mut expr);
        assert_matches!(ans, Err(_));

        // 1 * = Err
        let mut expr = vec![Token::Number(1.), Token::ProdOp('*')];
        let ans = eval_postfix(&mut expr);
        assert_matches!(ans, Err(_));

        // 1 + 1 + 1 = 3
        let mut expr = vec![Token::Number(1.), Token::Number(1.), Token::TermOp('+'), Token::Number(1.), Token::TermOp('+')];
        assert_eq!(Ok(3.0),eval_postfix(&mut expr));

        // Test forms
        // 4 + 2 * 3 = 8
        let mut expr = vec![
            Token::Number(4.),
            Token::Number(2.),
            Token::Number(3.),
            Token::ProdOp('*'),
            Token::TermOp('+')
        ];
        assert_eq!(Ok(10.0),eval_postfix(&mut expr));

        // 4 + 2 * 3 = 8
        let mut expr = vec![
            Token::Number(4.),
            Token::Number(3.),
            Token::Number(2.),
            Token::ProdOp('*'),
            Token::TermOp('+')
        ];
        assert_eq!(Ok(10.0),eval_postfix(&mut expr));

        // 4 + 2 * 3 = 8
        let mut expr = vec![
            Token::Number(2.),
            Token::Number(3.),
            Token::ProdOp('*'),
            Token::Number(4.),
            Token::TermOp('+')
        ];
        assert_eq!(Ok(10.0),eval_postfix(&mut expr));

        // 4 + 2 * 3 = 8
        let mut expr = vec![
            Token::Number(3.),
            Token::Number(2.),
            Token::ProdOp('*'),
            Token::Number(4.),
            Token::TermOp('+')
        ];
        assert_eq!(Ok(10.0),eval_postfix(&mut expr));
    }
}

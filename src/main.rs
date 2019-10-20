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
    Operator(usize, usize),
}


#[derive(Debug, Clone, Copy)]
enum Token {
    Number(f64),
    Operator(Operator),
    None,
}


#[derive(Debug, Clone, Copy)]
enum Operator {
    Add,
    Sub,
    Mul,
    Div,
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
            '*' | '/' | '+' | '-' => {
                indexes.push(TokenIndexes::Operator(i, i+1));
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
        tokens.push( 
            match index_pair {
                TokenIndexes::Number(a, b) => Token::Number(input[a..b].parse().unwrap()),
                TokenIndexes::Operator(a, b) => {
                    match input[a..b].parse().unwrap() {
                        '+' => Token::Operator(Operator::Add),
                        '-' => Token::Operator(Operator::Sub),
                        '*' => Token::Operator(Operator::Mul),
                        '/' => Token::Operator(Operator::Div),
                        _ => unimplemented!(),
                    }
                }
            }
        );
    }

    Ok(tokens)
}


/// Takes a Token vector in infix form and returns a Token vector in postfix form
fn to_postfix(tokens: &mut Vec<Token>) -> Vec<Token> {

    let mut calc_mem = CalcMem {
        main_stack: Vec::with_capacity(3),
        aux_stack: Vec::with_capacity(3),
        op_stack: Vec::with_capacity(3),
        current: Token::None,
    };

    let mut waiting_number = false;

    // Algorithm works as follows:
    // Read expression token by token
    for token in tokens.iter().rev() {
        calc_mem.current = *token;

        match calc_mem.current {

            // When token is a number, push it to the auxiliary stack
            // If an operator was waiting for it, push it as well.
            n @ Token::Number(_) => {
                calc_mem.aux_stack.push(n);

                if waiting_number {
                    waiting_number = false;
                    let op = calc_mem.op_stack.pop().unwrap();
                    calc_mem.aux_stack.push(op);
                }
            },

            // If the token is a mut/div operand, push to operand stack and wait for number
            Token::Operator(op) => {
                match op {

                    // If token is an addition or subtraction operator,
                    // then clear the auxiliary and operator stacks
                    // and push the operator onto the operator stack.
                    p @ Operator::Add | p @ Operator::Sub => {
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
                        calc_mem.op_stack.push(Token::Operator(p));
                    },

                    // If token is a multiplication or division operator,
                    // push it onto the operator stack and wait for a number
                    p @ Operator::Mul | p @ Operator::Div => {
                        calc_mem.op_stack.push(Token::Operator(p));
                        waiting_number = true;
                    },
                }
            },

            Token::None => unimplemented!(),
        };
    };

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
        Token::Operator(Operator::Add) => a + b,
        Token::Operator(Operator::Sub) => a - b,
        Token::Operator(Operator::Mul) => a * b,
        Token::Operator(Operator::Div) => {
            if b == 0.0 { return Err("Undefined: Division by zero"); }
            a / b
        },
        _ => unimplemented!(),
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
    let answer = eval_postfix(&mut tokens).unwrap();
    println!("= {}", answer);
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn eval_postfix_test() {

        // 1 + 1 = 2
        let mut expr = vec![Token::Number(1.), Token::Number(1.), Token::Operator(Operator::Add)];
        assert_eq!(Ok(2.0),eval_postfix(&mut expr));

        // 1 - 1 = 0
        let mut expr = vec![Token::Number(1.), Token::Number(1.), Token::Operator(Operator::Sub)];
        assert_eq!(Ok(0.0),eval_postfix(&mut expr));

        // 2 - 1 = 1
        let mut expr = vec![Token::Number(1.), Token::Number(2.), Token::Operator(Operator::Sub)];
        assert_eq!(Ok(1.0),eval_postfix(&mut expr));

        // 1 - 2 = -1
        let mut expr = vec![Token::Number(2.), Token::Number(1.), Token::Operator(Operator::Sub)];
        assert_eq!(Ok(-1.0),eval_postfix(&mut expr));

        // 1 * 1 = 1
        let mut expr = vec![Token::Number(1.), Token::Number(1.), Token::Operator(Operator::Mul)];
        assert_eq!(Ok(1.0),eval_postfix(&mut expr));

        // 2 * 1 = 2
        let mut expr = vec![Token::Number(2.), Token::Number(1.), Token::Operator(Operator::Mul)];
        assert_eq!(Ok(2.0),eval_postfix(&mut expr));

        // 2 * 0 = 0
        let mut expr = vec![Token::Number(2.), Token::Number(0.), Token::Operator(Operator::Mul)];
        assert_eq!(Ok(0.0),eval_postfix(&mut expr));

        // 1 / 1 = 1
        let mut expr = vec![Token::Number(1.), Token::Number(1.), Token::Operator(Operator::Div)];
        assert_eq!(Ok(1.0),eval_postfix(&mut expr));

        // 2 / 2 = 1
        let mut expr = vec![Token::Number(2.), Token::Number(2.), Token::Operator(Operator::Div)];
        assert_eq!(Ok(1.0),eval_postfix(&mut expr));

        // 2 / 1 = 2
        let mut expr = vec![Token::Number(1.), Token::Number(2.), Token::Operator(Operator::Div)];
        assert_eq!(Ok(2.0),eval_postfix(&mut expr));

        // 2 / 0 = Err
        let mut expr = vec![Token::Number(0.), Token::Number(2.), Token::Operator(Operator::Div)];
        let ans = eval_postfix(&mut expr);
        assert_matches!(ans, Err(_));

        // 1 * = Err
        let mut expr = vec![Token::Number(1.), Token::Operator(Operator::Mul)];
        let ans = eval_postfix(&mut expr);
        assert_matches!(ans, Err(_));

        // 1 + 1 + 1 = 3
        let mut expr = vec![Token::Number(1.), Token::Number(1.), Token::Operator(Operator::Add), Token::Number(1.), Token::Operator(Operator::Add)];
        assert_eq!(Ok(3.0),eval_postfix(&mut expr));

        // Test forms
        // 4 + 2 * 3 = 8
        let mut expr = vec![
            Token::Number(4.),
            Token::Number(2.),
            Token::Number(3.),
            Token::Operator(Operator::Mul),
            Token::Operator(Operator::Add)
        ];
        assert_eq!(Ok(10.0),eval_postfix(&mut expr));

        // 4 + 2 * 3 = 8
        let mut expr = vec![
            Token::Number(4.),
            Token::Number(3.),
            Token::Number(2.),
            Token::Operator(Operator::Mul),
            Token::Operator(Operator::Add)
        ];
        assert_eq!(Ok(10.0),eval_postfix(&mut expr));

        // 4 + 2 * 3 = 8
        let mut expr = vec![
            Token::Number(2.),
            Token::Number(3.),
            Token::Operator(Operator::Mul),
            Token::Number(4.),
            Token::Operator(Operator::Add)
        ];
        assert_eq!(Ok(10.0),eval_postfix(&mut expr));

        // 4 + 2 * 3 = 8
        let mut expr = vec![
            Token::Number(3.),
            Token::Number(2.),
            Token::Operator(Operator::Mul),
            Token::Number(4.),
            Token::Operator(Operator::Add)
        ];
        assert_eq!(Ok(10.0),eval_postfix(&mut expr));
    }
}

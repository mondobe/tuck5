use std::collections::HashMap;

use crate::meta::graph_with_tags;

use super::*;
use test_case::test_case;

pub fn parse(text: &str) -> Vec<Token<Vec<String>>> {
    meta::eval_prog_from_text(
        "
        % {
            {.print.}. print;
            {.let.}. let;
            {.true.}. true, expr;
            {.false.}. false, expr;
            {.if.}. if;
            {.input.}. input, expr;
            a..z | A..Z | _. letter;
            letter+. word, expr;

            0..9+. int, positive, number, expr;
            int & '.' & int+. decimal, positive, number, expr;
            '-' & positive. negative, number, expr;

            ws~;
        }

        {
            expr & '+' & expr: add, expr;
            print & expr & ';': printStmt, stmt;
            let & word & '=' & expr & ';': letStmt, stmt;
            word & '=' & expr & ';': mutStmt, stmt;
            if & expr & '{' & stmt+ & '}': ifStmt, stmt;
        }
    ",
        text,
    )
}

pub struct Program {
    prog: Vec<Statement>,
    errors: Vec<String>,
}

pub enum Statement {
    Let {
        var: String,
        val: Expression,
    },
    Mut {
        var: String,
        val: Expression,
    },
    If {
        cond: Expression,
        body: Vec<Statement>,
    },
    Print(Expression),
}

pub enum Expression {
    Number(f64),
    VarRef(String),
    Add(Box<Expression>, Box<Expression>),
    True,
    False,
    Input,
}

pub enum StackOp {
    Push(u64),
    Move(usize),
    Set(usize),
    Print,
    Add,
    If(Vec<StackOp>),
    Input,
}

pub fn eval_program(tokens: &Vec<Token<Vec<String>>>) -> Option<Program> {
    graph_with_tags(tokens);

    let mut to_ret = Program {
        prog: Vec::new(),
        errors: Vec::new(),
    };

    for token in tokens {
        if let Some(stmt) = eval_stmt(token) {
            to_ret.prog.push(stmt);
        } else {
            to_ret
                .errors
                .push("Tried to add statement, but could not.".to_string());
        }
    }

    Some(to_ret)
}

pub fn eval_stmt(token: &Token<Vec<String>>) -> Option<Statement> {
    if let TokenType::Branch(children) = &token.t_type {
        if token.data.contains(&"printStmt".to_string()) {
            Some(Statement::Print(eval_expr(&children[1])?))
        } else if token.data.contains(&"letStmt".to_string()) {
            Some(Statement::Let {
                var: children[1].content().to_string(),
                val: eval_expr(&children[3])?,
            })
        } else if token.data.contains(&"mutStmt".to_string()) {
            Some(Statement::Mut {
                var: children[0].content().to_string(),
                val: eval_expr(&children[2])?,
            })
        } else if token.data.contains(&"ifStmt".to_string()) {
            Some(Statement::If {
                cond: eval_expr(&children[1])?,
                body: children[3..children.len() - 1]
                    .iter()
                    .map(|t| eval_stmt(t))
                    .filter_map(|o| o)
                    .collect(),
            })
        } else {
            None
        }
    } else {
        None
    }
}

pub fn eval_expr(token: &Token<Vec<String>>) -> Option<Expression> {
    if token.data.contains(&"number".to_string()) {
        Some(Expression::Number(str::parse::<f64>(token.content()).ok()?))
    } else if token.data.contains(&"word".to_string()) {
        Some(Expression::VarRef(token.content().to_string()))
    } else if token.data.contains(&"add".to_string()) {
        if let TokenType::Branch(children) = &token.t_type {
            Some(Expression::Add(
                Box::new(eval_expr(&children[0])?),
                Box::new(eval_expr(&children[2])?),
            ))
        } else {
            None
        }
    } else if token.data.contains(&"true".to_string()) {
        Some(Expression::True)
    } else if token.data.contains(&"false".to_string()) {
        Some(Expression::False)
    } else if token.data.contains(&"input".to_string()) {
        Some(Expression::Input)
    } else {
        None
    }
}

pub fn prog_to_stack(prog: &Program) -> Vec<StackOp> {
    let mut vars: HashMap<&str, usize> = HashMap::new();
    let mut ops = Vec::new();
    let mut stack_length = 0usize;
    for err in &prog.errors {
        println!("ERROR: {err}");
    }
    for stmt in &prog.prog {
        ops.append(&mut stmt_on_stack(stmt, &mut vars, &mut stack_length));
    }
    ops
}

pub fn stmt_on_stack<'a>(
    stmt: &'a Statement,
    vars: &mut HashMap<&'a str, usize>,
    stack_length: &mut usize,
) -> Vec<StackOp> {
    let mut ops = Vec::new();
    match stmt {
        Statement::Let { var, val } => {
            eval_to_stack(val, &mut ops, &vars, stack_length);
            vars.insert(&var, *stack_length - 1);
        }
        Statement::Mut { var, val } => {
            eval_to_stack(val, &mut ops, &vars, stack_length);
            ops.push(StackOp::Set(*vars.get(var.as_str()).unwrap()))
        }
        Statement::Print(var) => {
            eval_to_stack(var, &mut ops, &vars, stack_length);
            ops.push(StackOp::Print);
            *stack_length += 1;
        }
        Statement::If { cond, body } => {
            eval_to_stack(cond, &mut ops, &vars, stack_length);
            ops.push(StackOp::If(
                body.iter()
                    .map(|s| stmt_on_stack(s, vars, stack_length))
                    .flatten()
                    .collect(),
            ))
        }
    }
    ops
}

pub fn eval_to_stack(
    expr: &Expression,
    ops: &mut Vec<StackOp>,
    vars: &HashMap<&str, usize>,
    stack_length: &mut usize,
) {
    match expr {
        Expression::Number(num) => {
            ops.push(StackOp::Push(num.to_bits()));
            *stack_length += 1;
        }
        Expression::VarRef(var) => {
            ops.push(StackOp::Move(vars[var.as_str()]));
            *stack_length += 1;
        }
        Expression::Add(lhs, rhs) => {
            eval_to_stack(lhs, ops, vars, stack_length);
            eval_to_stack(rhs, ops, vars, stack_length);
            ops.push(StackOp::Add);
            *stack_length -= 1;
        }
        Expression::True => {
            ops.push(StackOp::Push(1));
            *stack_length += 1;
        }
        Expression::False => {
            ops.push(StackOp::Push(0));
            *stack_length += 1;
        }
        Expression::Input => {
            ops.push(StackOp::Input);
            *stack_length += 1;
        }
    }
}

pub fn run_stack_ops(stack: &mut Vec<u64>, ops: &Vec<StackOp>, verbose: bool) {
    for op in ops {
        match op {
            StackOp::Move(ptr) => {
                if verbose {
                    println!("Pushing element at {ptr}");
                }
                stack.push(stack[*ptr]);
            }
            StackOp::Set(ptr) => {
                let val = stack.pop().unwrap();
                if verbose {
                    println!("Setting {ptr} to {val}");
                }
                stack[*ptr] = val;
            }
            StackOp::Print => println!("{}", f64::from_bits(stack.pop().unwrap())),
            StackOp::Push(val) => {
                if verbose {
                    println!("Pushing {val}");
                }
                stack.push(*val);
            }
            StackOp::Add => {
                let rhs = f64::from_bits(stack.pop().unwrap());
                let lhs = f64::from_bits(stack.pop().unwrap());
                if verbose {
                    println!("Adding {lhs} + {rhs}");
                }
                stack.push((lhs + rhs).to_bits());
            }
            StackOp::If(inner) => {
                if stack.pop().unwrap() == 1 {
                    run_stack_ops(stack, inner, verbose)
                }
            }
            StackOp::Input => {
                let mut buf = String::new();
                let _ = std::io::stdin().read_line(&mut buf);
                stack.push(str::parse::<f64>(&buf.trim()).unwrap().to_bits());
            }
        }
    }
}

#[test_case("
print 3;
"; "simple print statement")]
#[test_case("
print 03.06;
"; "decimal print statement")]
#[test_case("
let a = 3;
print a;
"; "simple let statement")]
#[test_case("
let a = 3;
let b = a;
print b;
"; "double let statement")]
#[test_case("
let a = 3;
let b = a;
print b;
print a;
"; "double print statement")]
#[test_case("
print 3 + 2;
"; "simple add expression")]
#[test_case("
print 1 + 2 + 3 + 4;
"; "quadruple add expression")]
#[test_case("
let a = 3;
let b = a;
print a + b;
"; "add var to itself")]
#[test_case("
let a = 3;
a = 4;
print a;
"; "simple mut statement")]
#[test_case("
let a = 1;
a = a + a;
print a;
"; "self-incrementing")]
#[test_case("
if true {
    print 5;
}
if false {
    print 6;
}
"; "simple if expression")]
#[test_case("
if false {
    let a = 3;
}
let b = 5;
let c = 7;
print a;
"; "let in if")]
#[test_case("
print input + 1;
"; "echo input")]
pub fn run_prog_from_text(text: &str) {
    let stack = &mut Vec::new();
    run_stack_ops(
        stack,
        &prog_to_stack(&eval_program(&parse(&text)).unwrap()),
        false,
    )
}

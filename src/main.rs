mod ir;
use std::collections::HashMap;
use parse::*;
pub mod parse;

fn main() {
    let filename = std::env::args().nth(1).expect("name of input file should be supplied as first argument");

    let sourcecode: String = std::fs::read_to_string(filename).expect("failed to read file");

    let (_, code ) = Ast::parse_ws(&sourcecode).expect("failed to parse sourcecode");

    let scope = eval(&code, HashMap::new());

    for (key, value) in scope {
        println!("{}: {}", key, value);
    }
}

// Evaluate AST
fn eval(code: &Ast, mut scope: HashMap<String, u64>) -> HashMap<String, u64> {
    for statement in &code.statements[..] {
        match statement {
            Statement::If(st) => scope = eval_if(st, scope),
            Statement::Loop(st) => scope = eval_loop(st, scope),
            Statement::Assignment(st) => scope = eval_assignment(st, scope),
        }
    }

    scope
}

fn eval_assignment(code: &Assignment, mut scope: HashMap<String, u64>) -> HashMap<String, u64> {
    let l = *scope.get(&code.left_hand_side.name).unwrap_or(&0);
    let r = code.right_hand_side.value;

    let value = match code.operation {
        Operation::Add => l + r,
        Operation::Sub => l - r,
        Operation::Mul => l * r,
        Operation::Div => l / r,
        Operation::Mod => l % r,
    };

    scope.insert(code.destination.name.clone(), value);

    scope
}

fn eval_if(code: &If, mut scope: HashMap<String, u64>) -> HashMap<String, u64> {
    let l = *scope.get(&code.variable.name).unwrap_or(&0);

    let condition = match code.condition {
        Condition::Eq(Constant{value}) => l == value,
        Condition::Neq(Constant{value}) => l == value,
    };

    if condition {
        scope = eval(&code.instructions, scope);
    }

    scope
}

fn eval_loop(code: &Loop, scope: HashMap<String, u64>) -> HashMap<String, u64> {
    let counter = *scope.get(&code.counter.name).unwrap_or(&0);

    (0..counter).fold(scope,|scope, _| eval(&code.instruction, scope))
}



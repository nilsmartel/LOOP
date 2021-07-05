pub use compilation::compile;

#[derive(Clone)]
pub struct Program {
    pub variables: Vec<String>,
    pub instructions: Vec<Instruction>,
}

#[derive(Clone)]
pub enum Instruction {
    Assign {
        to: String,
        expr: Expr,
    },
    If {
        condition: Expr,
        body: Vec<Instruction>,
    },
    Loop {
        times: Value,
        body: Vec<Instruction>,
    },
}

#[derive(Clone)]
pub struct Expr {
    pub left: Value,
    pub right: Value,
    pub op: Operation,
}

#[derive(Clone, Copy)]
pub enum Operation {
    Plus,
    Minus,
    Times,
    Divided,
    Modulo,
    Equal,
    NotEqual,
}

#[derive(Clone)]
pub enum Value {
    Variable(String),
    Constant(i64),
}

mod compilation {
    use std::collections::HashMap;

    use cranelift::prelude::{types::I64, AbiParam, EntityRef, ExternalName, InstBuilder, IntCC};
    use cranelift_codegen::{
        binemit::{NullStackMapSink, NullTrapSink},
        ir::Value,
    };
    use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
    use cranelift_module::{default_libcall_names, Module};
    use super::Program;

    use cranelift_jit::{JITBuilder, JITModule};

    pub fn compile(program: Program) -> fn(i64) -> i64 {
        let mut module = {
            let builder = JITBuilder::new(default_libcall_names());
            JITModule::new(builder)
        };
        let sign = {
            let mut sign = module.make_signature();
            sign.params.push(AbiParam::new(I64));
            sign.returns.push(AbiParam::new(I64));
            sign
        };
        let func_id = module.declare_anonymous_function(&sign).unwrap();
        let mut context = module.make_context();
        context.func.signature = sign;
        context.func.name = ExternalName::User {
            namespace: 0,
            index: func_id.as_u32(),
        };
        {
            let mut fctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut context.func, &mut fctx);

            let entry = builder.create_block();
            builder.append_block_params_for_function_params(entry);
            builder.switch_to_block(entry);

            let mut varcount = program.variables.len();
            let vars: HashMap<String, Variable> = program
                .variables
                .into_iter()
                .zip(0usize..)
                .map(|(name, i)| {
                    let var = Variable::new(i);

                    builder.declare_var(var, I64);
                    let zero = builder.ins().iconst(I64, 0);
                    builder.def_var(var,zero);
                    (name, var)
                })
                .collect();


            // Set variable input to argument of function
            if let Some(var) = vars.get("input") {
                builder.def_var(*var, builder.block_params(entry)[0]);
            }

            jit(&mut builder, &vars, &program.instructions, &mut varcount);

            // return value stored in output OR 0 if output is not set
            let retval = if let Some(var) = vars.get("output") {
                builder.use_var(*var)
            } else {
                builder.ins().iconst(I64, 0)
            };
            builder.ins().return_(&[retval]);

            // finish up
            builder.seal_all_blocks();
            builder.finalize();
        }

        // start the actual jit compilation
        let mut trap_sink = NullTrapSink {};
        let mut stack_map_sink = NullStackMapSink {};
        module
            .define_function(func_id, &mut context, &mut trap_sink, &mut stack_map_sink)
            .unwrap();
        module.clear_context(&mut context);
        module.finalize_definitions();
        let ptr = module.get_finalized_function(func_id);

        unsafe { std::mem::transmute::<*const u8, fn(i64) -> i64>(ptr) }
    }

    fn jit(
        mut builder: &mut FunctionBuilder,
        vars: &HashMap<String, Variable>,
        instructions: &[super::Instruction],
        mut varcount: &mut usize,
    ) {
        if instructions.len() == 0 {
            return;
        }

        let instr = &instructions[0];
        let instructions = &instructions[1..];

        use super::Instruction::*;
        match instr {
            &Assign { ref to, ref expr } => {
                let var = vars.get(to).unwrap().clone();
                let value = eval(expr, &mut builder, &vars);
                builder.def_var(var, value);
            }
            &If {
                ref condition,
                ref body,
            } => {
                let condition = eval(condition, &mut builder, &vars);
                let ifblock = builder.create_block();
                let continueblock = builder.create_block();

                // TODO 0 or 1 here? -> verify what the result of an comparision is
                let success = builder.ins().iconst(I64, 1);

                builder
                    .ins()
                    .br_icmp(IntCC::Equal, condition, success, ifblock, &[]);
                builder.ins().jump(continueblock, &[]);
                // TODO? seal current block

                builder.switch_to_block(ifblock);
                jit(&mut builder, vars, body, varcount);
                builder.seal_block(ifblock);

                // Progress to the next block
                builder.switch_to_block(continueblock);
            }
            &Loop {
                ref times,
                ref body,
            } => {
                let repetitions = val(&times, &mut builder, &vars);

                let counter = Variable::new(*varcount);
                *varcount += 1;
                builder.declare_var(counter, I64);
                builder.def_var(counter, repetitions);

                let loopblock = builder.create_block();
                let continueblock = builder.create_block();

                builder.ins().jump(loopblock, &[]);
                // TODO seal current block?
                builder.switch_to_block(loopblock);
                let counter_value = builder.use_var(counter);
                let zero = builder.ins().iconst(I64, 0);
                builder.ins().br_icmp(
                    IntCC::SignedLessThanOrEqual,
                    counter_value,
                    zero,
                    continueblock,
                    &[],
                );
                // emit loop body
                jit(&mut builder, vars, body, &mut varcount);
                // jump back to start of loop
                builder.ins().jump(loopblock, &[]);

                builder.seal_block(loopblock);
                builder.switch_to_block(continueblock);
            }
        }

        jit(builder, vars, instructions, &mut varcount)
    }

    fn eval(
        expr: &super::Expr,
        mut builder: &mut FunctionBuilder,
        vars: &HashMap<String, Variable>,
    ) -> Value {
        let left = val(&expr.left, &mut builder, vars);
        let right = val(&expr.right, &mut builder, vars);

        use super::Operation::*;
        match expr.op {
            Plus => builder.ins().iadd(left, right),
            Minus => builder.ins().isub(left, right),
            Times => builder.ins().imul(left, right),
            // TODO can this right?
            Divided => builder.ins().udiv(left, right),
            Equal => builder.ins().icmp(IntCC::Equal, left, right),
            NotEqual => builder.ins().icmp(IntCC::NotEqual, left, right),
            Modulo => unimplemented!(),
        }
    }

    fn val(
        value: &super::Value,
        builder: &mut FunctionBuilder,
        vars: &HashMap<String, Variable>,
    ) -> Value {
        match value {
            &super::Value::Constant(n) => builder.ins().iconst(I64, n),
            &super::Value::Variable(ref name) => {
                let var = vars[name];
                builder.use_var(var)
            }
        }
    }
}

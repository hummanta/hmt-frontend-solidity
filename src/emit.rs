// Copyright (c) The Hummanta Authors. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;

use cranelift::{
    module::FuncId,
    object::ObjectModule,
    prelude::{types::I64, EntityRef, FunctionBuilder, InstBuilder, Value, Variable},
};

use crate::ast::*;

pub struct EmitContext<'a> {
    pub module: &'a mut ObjectModule,
    pub builder: FunctionBuilder<'a>,
    pub functions: HashMap<String, FuncId>,
    pub variables: HashMap<String, Variable>,
    pub index: usize,
}

impl<'a> EmitContext<'a> {
    pub fn new(module: &'a mut ObjectModule, builder: FunctionBuilder<'a>) -> Self {
        Self { module, builder, functions: HashMap::new(), variables: HashMap::new(), index: 0 }
    }

    pub fn declare_var(&mut self, name: &str) -> Variable {
        let var = Variable::new(self.index);
        self.index += 1;
        self.variables.insert(name.to_string(), var);
        var
    }

    pub fn get_variable(&self, name: &str) -> Option<Variable> {
        self.variables.get(name).cloned()
    }
}

pub struct CraneliftEmitter<'a> {
    ctx: &'a mut EmitContext<'a>,
}

impl<'a> CraneliftEmitter<'a> {
    pub fn new(ctx: &'a mut EmitContext<'a>) -> Self {
        Self { ctx }
    }
}

impl Visitor<Value> for CraneliftEmitter<'_> {
    fn visit_program(&mut self, program: &Program) -> Value {
        let entry = self.ctx.builder.create_block();
        self.ctx.builder.switch_to_block(entry);

        for stmt in program.iter() {
            stmt.accept(self);
        }

        self.ctx.builder.ins().return_(&[]);
        Value::new(0)
    }

    fn visit_statement(&mut self, stmt: &Statement) -> Value {
        match stmt {
            Statement::Variable(stmt) => stmt.accept(self),
            Statement::Print(stmt) => stmt.accept(self),
        }
    }

    fn visit_var_statement(&mut self, stmt: &VarStatement) -> Value {
        let val = stmt.value.accept(self);
        let var = self.ctx.declare_var(&stmt.name);
        self.ctx.builder.declare_var(var, I64);
        self.ctx.builder.def_var(var, val);
        val
    }

    fn visit_print_statement(&mut self, stmt: &PrintStatement) -> Value {
        stmt.value.accept(self)
    }

    fn visit_expression(&mut self, expr: &Expression) -> Value {
        match expr {
            Expression::Integer(num) => self.ctx.builder.ins().iconst(I64, *num),
            Expression::Variable(name) => {
                let var = self
                    .ctx
                    .get_variable(name)
                    .unwrap_or_else(|| panic!("Undefined variable: {}", name));
                self.ctx.builder.use_var(var)
            }
            Expression::BinaryOperation { lhs, operator, rhs } => {
                let (lhs, rhs) = (lhs.accept(self), rhs.accept(self));

                match operator {
                    Operator::Add => self.ctx.builder.ins().iadd(lhs, rhs),
                    Operator::Sub => self.ctx.builder.ins().isub(lhs, rhs),
                    Operator::Mul => self.ctx.builder.ins().imul(lhs, rhs),
                    Operator::Div => self.ctx.builder.ins().sdiv(lhs, rhs),
                }
            }
        }
    }
}

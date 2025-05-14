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
    prelude::{EntityRef, FunctionBuilder, InstBuilder, Value, Variable},
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

        for unit in program.iter() {
            unit.accept(self);
        }

        self.ctx.builder.ins().return_(&[]);
        Value::new(0)
    }

    fn visit_source_unit(&mut self, source_unit: &SourceUnit) -> Value {
        match source_unit {
            SourceUnit::PragmaDirective(pragma) => pragma.accept(self),
            SourceUnit::ContractDefinition(contract) => contract.accept(self),
            SourceUnit::VariableDefinition(var) => var.accept(self),
            SourceUnit::StraySemicolon => Value::from_u32(0),
        }
    }

    fn visit_pragma(&mut self, _pragma: &PragmaDirective) -> Value {
        todo!()
    }

    fn visit_contract(&mut self, _contract: &ContractDefinition) -> Value {
        todo!()
    }

    fn visit_contract_part(&mut self, part: &ContractPart) -> Value {
        match part {
            ContractPart::VariableDefinition(var) => var.accept(self),
            ContractPart::StraySemicolon => Value::from_u32(0),
        }
    }

    fn visit_variable(&mut self, _var: &VariableDefinition) -> Value {
        todo!()
    }

    fn visit_expression(&mut self, _exp: &Expression) -> Value {
        todo!()
    }
}

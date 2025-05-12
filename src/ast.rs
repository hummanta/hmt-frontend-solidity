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

#[derive(Clone, Debug)]
pub struct Program(pub Vec<Statement>);

impl Program {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_program(self)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Statement> {
        self.0.iter()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Variable(VarStatement),
    Print(PrintStatement),
}

impl Statement {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_statement(self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarStatement {
    pub name: String,
    pub value: Box<Expression>,
}

impl VarStatement {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_var_statement(self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrintStatement {
    pub value: Box<Expression>,
}

impl PrintStatement {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_print_statement(self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Integer(i64),
    Variable(String),
    BinaryOperation { lhs: Box<Expression>, operator: Operator, rhs: Box<Expression> },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

impl Expression {
    pub fn accept<T, V: Visitor<T>>(&self, visitor: &mut V) -> T {
        visitor.visit_expression(self)
    }
}

pub trait Visitor<T> {
    fn visit_program(&mut self, program: &Program) -> T;
    fn visit_statement(&mut self, stmt: &Statement) -> T;
    fn visit_var_statement(&mut self, stmt: &VarStatement) -> T;
    fn visit_print_statement(&mut self, stmt: &PrintStatement) -> T;
    fn visit_expression(&mut self, expr: &Expression) -> T;
}

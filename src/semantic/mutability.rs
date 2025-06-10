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

use crate::{
    diagnostics::{Diagnostic, Diagnostics, Level},
    helpers::CodeLocation,
    parser::ast as pt,
    semantic::{
        ast::{
            Builtin, CallTy, DestructureField, Expression, Function, Mutability, Recurse,
            RetrieveType, Statement, Type,
        },
        context::Context,
    },
};
use bitflags::bitflags;

/// Check state mutability
pub fn check(ctx: &mut Context, no: usize) {
    if !ctx.diagnostics.any_errors() {
        for func in &ctx.functions {
            if func.loc_prototype.try_no() != Some(no) || func.ty == pt::FunctionTy::Modifier {
                continue;
            }

            let diagnostics = check_mutability(func, ctx);

            ctx.diagnostics.extend(diagnostics);
        }
    }
}

/// While we recurse through the AST, maintain some state
struct StateCheck<'a> {
    diagnostic: Diagnostics,
    declared_access: Access,
    required_access: Access,
    func: &'a Function,
    modifier: Option<pt::Loc>,
    ctx: &'a Context,
    data_account: DataAccountUsage,
}

impl StateCheck<'_> {
    fn value(&mut self, loc: &pt::Loc) {
        self.check_level(loc, Access::Value);
        self.required_access.increase_to(Access::Value);
    }

    fn write(&mut self, loc: &pt::Loc) {
        self.check_level(loc, Access::Write);
        self.required_access.increase_to(Access::Write);
    }

    fn read(&mut self, loc: &pt::Loc) {
        self.check_level(loc, Access::Read);
        self.required_access.increase_to(Access::Read);
    }

    /// Compare the declared access level to the desired access level.
    /// If there is an access violation, it'll be reported to the diagnostics.
    fn check_level(&mut self, loc: &pt::Loc, desired: Access) {
        if self.declared_access >= desired {
            return;
        }

        let (message, note) = match desired {
            Access::Read => ("reads from state", "read to state"),
            Access::Write => ("writes to state", "write to state"),
            Access::Value => (
                "accesses value sent, which is only allowed for payable functions",
                "access of value sent",
            ),
            Access::None => unreachable!("desired access can't be None"),
        };

        let diagnostic = self
            .modifier
            .map(|modifier_loc| {
                let message = format!(
                    "function declared '{}' but modifier {}",
                    self.func.mutability, message
                );
                Diagnostic::builder(modifier_loc, Level::Error)
                    .message(message)
                    .note(*loc, note)
                    .build()
            })
            .unwrap_or_else(|| {
                let message = format!(
                    "function declared '{}' but this expression {}",
                    self.func.mutability, message
                );
                Diagnostic::error(*loc, message)
            });

        self.diagnostic.push(diagnostic);
    }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd)]
enum Access {
    None,
    Read,
    Write,
    Value,
}

bitflags! {
    #[derive(PartialEq, Eq, Copy, Clone, Debug)]
    struct DataAccountUsage: u8 {
        const NONE = 0;
        const READ = 1;
        const WRITE = 2;
    }
}

impl Access {
    fn increase_to(&mut self, other: Access) {
        if *self < other {
            *self = other;
        }
    }
}

fn check_mutability(func: &Function, ctx: &Context) -> Diagnostics {
    if func.is_virtual {
        return Default::default();
    }

    let mut state = StateCheck {
        diagnostic: Default::default(),
        declared_access: match func.mutability {
            Mutability::Pure(_) => Access::None,
            Mutability::View(_) => Access::Read,
            Mutability::Nonpayable(_) => Access::Write,
            Mutability::Payable(_) => Access::Value,
        },
        required_access: Access::None,
        func,
        modifier: None,
        ctx,
        data_account: DataAccountUsage::NONE,
    };

    for arg in &func.modifiers {
        if let Expression::InternalFunctionCall { function, args, .. } = &arg {
            // check the arguments to the modifiers
            for arg in args {
                arg.recurse(&mut state, read_expression);
            }

            let contract_no = func.contract_no.expect("only functions in contracts have modifiers");

            // check the modifier itself
            if let Expression::InternalFunction { function_no, signature, .. } = function.as_ref() {
                let function_no = if let Some(signature) = signature {
                    state.ctx.contracts[contract_no].virtual_functions[signature]
                        .last()
                        .copied()
                        .unwrap()
                } else {
                    *function_no
                };

                // modifiers do not have mutability, bases or modifiers itself
                let func = &ctx.functions[function_no];

                state.modifier = Some(arg.loc());

                recurse_statements(&func.body, &mut state);

                state.modifier = None;
            }
        }
    }

    recurse_statements(&func.body, &mut state);

    if pt::FunctionTy::Function == func.ty && !func.is_accessor {
        if state.required_access == Access::None {
            match func.mutability {
                Mutability::Payable(_) | Mutability::Pure(_) => (),
                Mutability::Nonpayable(_) => {
                    state.diagnostic.push(Diagnostic::warning(
                        func.loc_prototype,
                        "function can be declared 'pure'",
                    ));
                }
                _ => {
                    state.diagnostic.push(Diagnostic::warning(
                        func.loc_prototype,
                        format!("function declared '{}' can be declared 'pure'", func.mutability),
                    ));
                }
            }
        }

        // don't suggest marking payable as view (declared_access == Value)
        if state.required_access == Access::Read && state.declared_access == Access::Write {
            state
                .diagnostic
                .push(Diagnostic::warning(func.loc_prototype, "function can be declared 'view'"));
        }
    }

    state.diagnostic
}

fn recurse_statements(stmts: &[Statement], state: &mut StateCheck) {
    for stmt in stmts.iter() {
        match stmt {
            Statement::Block { statements, .. } => {
                recurse_statements(statements, state);
            }
            Statement::VariableDecl(_, _, _, Some(expr)) => {
                expr.recurse(state, read_expression);
            }
            Statement::VariableDecl(_, _, _, None) => (),
            Statement::If(_, _, expr, then_, else_) => {
                expr.recurse(state, read_expression);
                recurse_statements(then_, state);
                recurse_statements(else_, state);
            }
            Statement::DoWhile(_, _, body, expr) | Statement::While(_, _, expr, body) => {
                expr.recurse(state, read_expression);
                recurse_statements(body, state);
            }
            Statement::For { init, cond, next, body, .. } => {
                recurse_statements(init, state);
                if let Some(cond) = cond {
                    cond.recurse(state, read_expression);
                }
                if let Some(next) = next {
                    next.recurse(state, read_expression);
                }
                recurse_statements(body, state);
            }
            Statement::Expression(_, _, expr) => {
                expr.recurse(state, read_expression);
            }
            Statement::Delete(loc, _, _) => {
                state.data_account |= DataAccountUsage::WRITE;
                state.write(loc)
            }
            Statement::Destructure(_, fields, expr) => {
                // This is either a list or internal/external function call
                expr.recurse(state, read_expression);

                for field in fields {
                    if let DestructureField::Expression(expr) = field {
                        expr.recurse(state, write_expression);
                    }
                }
            }
            Statement::Return(_, None) => {}
            Statement::Return(_, Some(expr)) => {
                expr.recurse(state, read_expression);
            }
            Statement::TryCatch(_, _, try_catch) => {
                try_catch.expr.recurse(state, read_expression);
                recurse_statements(&try_catch.ok_stmt, state);
                for clause in &try_catch.errors {
                    recurse_statements(&clause.stmt, state);
                }
                if let Some(clause) = try_catch.catch_all.as_ref() {
                    recurse_statements(&clause.stmt, state);
                }
            }
            Statement::Emit { loc, .. } => state.write(loc),
            Statement::Revert { args, .. } => {
                for arg in args {
                    arg.recurse(state, read_expression);
                }
            }
            Statement::Break(_) | Statement::Continue(_) | Statement::Underscore(_) => (),
        }
    }
}

fn read_expression(expr: &Expression, state: &mut StateCheck) -> bool {
    match expr {
        Expression::StorageLoad { loc, .. } => {
            state.data_account |= DataAccountUsage::READ;
            state.read(loc)
        }
        Expression::PreIncrement { expr, .. } |
        Expression::PreDecrement { expr, .. } |
        Expression::PostIncrement { expr, .. } |
        Expression::PostDecrement { expr, .. } => {
            expr.recurse(state, write_expression);
        }
        Expression::Assign { left, right, .. } => {
            right.recurse(state, read_expression);
            left.recurse(state, write_expression);
            return false;
        }
        Expression::StorageArrayLength { loc, .. } => {
            state.data_account |= DataAccountUsage::READ;
            state.read(loc);
        }
        Expression::StorageVariable { loc, .. } => {
            state.data_account |= DataAccountUsage::READ;
            state.read(loc);
        }
        Expression::Builtin { kind: Builtin::FunctionSelector, args, .. } => {
            if let Expression::ExternalFunction { .. } = &args[0] {
                // in the case of `this.func.selector`, the address of this is not used and
                // therefore does not read state. Do not recurse down the `address` field of
                // Expression::ExternalFunction
                return false;
            }
        }
        Expression::Builtin {
            loc,
            kind:
                Builtin::GetAddress |
                Builtin::BlockNumber |
                Builtin::Slot |
                Builtin::Timestamp |
                Builtin::BlockCoinbase |
                Builtin::BlockDifficulty |
                Builtin::BlockHash |
                Builtin::Sender |
                Builtin::Origin |
                Builtin::Gasleft |
                Builtin::Gasprice |
                Builtin::GasLimit |
                Builtin::MinimumBalance |
                Builtin::Balance |
                Builtin::Accounts |
                Builtin::ContractCode,
            ..
        } => state.read(loc),

        Expression::Builtin {
            loc,
            kind: Builtin::PayableSend | Builtin::PayableTransfer | Builtin::SelfDestruct,
            ..
        } => state.write(loc),
        Expression::Builtin { loc, kind: Builtin::Value, .. } => {
            // internal/private functions cannot be declared payable, so msg.value is only checked
            // as reading state in private/internal functions in solc.
            if state.func.is_public() {
                state.value(loc)
            } else {
                state.read(loc)
            }
        }
        Expression::Builtin { loc, kind: Builtin::ArrayPush | Builtin::ArrayPop, args, .. }
            if args[0].ty().is_contract_storage() =>
        {
            state.data_account |= DataAccountUsage::WRITE;
            state.write(loc)
        }

        Expression::Constructor { loc, .. } => {
            state.write(loc);
        }
        Expression::ExternalFunctionCall { loc, function, .. } |
        Expression::InternalFunctionCall { loc, function, .. } => match function.ty() {
            Type::ExternalFunction { mutability, .. } |
            Type::InternalFunction { mutability, .. } => {
                match mutability {
                    Mutability::Nonpayable(_) | Mutability::Payable(_) => state.write(loc),
                    Mutability::View(_) => state.read(loc),
                    Mutability::Pure(_) => (),
                };
            }
            _ => unreachable!(),
        },
        Expression::ExternalFunctionCallRaw { loc, ty, .. } => match ty {
            CallTy::Static => state.read(loc),
            CallTy::Delegate | CallTy::Regular => state.write(loc),
        },
        _ => (),
    }
    true
}

fn write_expression(expr: &Expression, state: &mut StateCheck) -> bool {
    match expr {
        Expression::StructMember { loc, expr: array, .. } |
        Expression::Subscript { loc, array, .. } => {
            if array.ty().is_contract_storage() {
                state.data_account |= DataAccountUsage::WRITE;
                state.write(loc);
                return false;
            }
        }
        Expression::Variable { loc, ty, var_no: _ } => {
            if ty.is_contract_storage() && !expr.ty().is_contract_storage() {
                state.data_account |= DataAccountUsage::WRITE;
                state.write(loc);
                return false;
            }
        }
        Expression::StorageVariable { loc, .. } => {
            state.data_account |= DataAccountUsage::WRITE;
            state.write(loc);
            return false;
        }
        Expression::Builtin { loc, kind: Builtin::Accounts, .. } => {
            state.write(loc);
        }
        _ => (),
    }

    true
}

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

use thiserror::Error;

use super::{context::Context, visitor::SemanticVisitor};

use crate::{
    diagnostics::{Diagnostic, Diagnostics, Level, Note},
    helpers::CodeLocation,
    parser::{
        ast::{self as pt},
        visitor::Visitor,
    },
    semantic::{
        ast::{Expression, Mutability, Type, Using, UsingFunction, UsingList},
        context::ResolveTypeContext,
        visitor::SemanticVisitable,
    },
};

/// Resolve the global using directives
pub struct UsingResolver<'a> {
    /// Shared context for diagnostics and state
    ctx: &'a mut Context,
    no: usize,
    contract_no: Option<usize>,
    using: Option<Using>,
}

impl<'a> UsingResolver<'a> {
    /// Creates a new using resolver with the given context
    pub fn new(ctx: &'a mut Context, no: usize, contract_no: Option<usize>) -> Self {
        Self { ctx, no, contract_no, using: None }
    }

    fn resolve_library(
        &mut self,
        library: &pt::IdentifierPath,
        diagnostics: &mut Diagnostics,
    ) -> Result<UsingList, UsingResolverError> {
        if let Ok(library_no) =
            self.ctx.resolve_contract_with_namespace(self.no, library, diagnostics)
        {
            if self.ctx.contracts[library_no].is_library() {
                Ok(UsingList::Library(library_no))
            } else {
                self.ctx.diagnostics.push(Diagnostic::error(
                    library.loc,
                    format!(
                        "library expected but {} '{}' found",
                        self.ctx.contracts[library_no].ty, library
                    ),
                ));
                Err(UsingResolverError::Noops)
            }
        } else {
            self.ctx.diagnostics.extend(diagnostics.clone());
            Err(UsingResolverError::Noops)
        }
    }

    fn resove_functions(
        &mut self,
        using: &pt::Using,
        functions: &Vec<pt::UsingFunction>,
        ty: &Option<Type>,
        diagnostics: &mut Diagnostics,
    ) -> Result<UsingList, UsingResolverError> {
        let mut res = Vec::new();

        for using_function in functions {
            let function_name = &using_function.path;
            if let Ok(list) = self.ctx.resolve_function_with_namespace(
                self.no,
                self.contract_no,
                &using_function.path,
                diagnostics,
            ) {
                if list.len() > 1 {
                    let notes = list
                        .iter()
                        .map(|(loc, _)| Note {
                            loc: *loc,
                            message: format!("definition of '{function_name}'"),
                        })
                        .collect();

                    diagnostics.push(
                        Diagnostic::builder(function_name.loc, Level::Error)
                            .message(format!("'{function_name}' is an overloaded function"))
                            .notes(notes)
                            .build(),
                    );
                    continue;
                }

                let (loc, func_no) = list[0];

                let func = &self.ctx.functions[func_no];

                if let Some(contract_no) = func.contract_no {
                    if !self.ctx.contracts[contract_no].is_library() {
                        diagnostics.push(
                            Diagnostic::builder(function_name.loc, Level::Error)
                                .message(format!("'{function_name}' is not a library function"))
                                .note(
                                    func.loc_prototype,
                                    format!("definition of {}", using_function.path),
                                )
                                .build(),
                        );
                        continue;
                    }
                }

                if func.params.is_empty() {
                    diagnostics.push(
                        Diagnostic::builder(function_name.loc, Level::Error)
                            .message(format!("'{function_name}' has no arguments. At least one argument required"))
                            .note(loc, format!("definition of '{function_name}'"))
                            .build(),
                    );
                    continue;
                }

                let oper = if let Some(mut oper) = using_function.oper {
                    if self.contract_no.is_some() || using.global.is_none() || ty.is_none() {
                        diagnostics.push(Diagnostic::error(
                            using_function.loc,
                            "user defined operator can only be set in a global 'using for' directive",
                        ));
                        break;
                    }

                    let ty = ty.as_ref().unwrap();

                    if !matches!(*ty, Type::UserType(_)) {
                        diagnostics.push(Diagnostic::error(
                            using_function.loc,
                            format!("user defined operator can only be used with user defined types. Type {} not permitted", ty.to_string(self.ctx))
                        ));
                        break;
                    }

                    // The '-' operator may be for subtract or negation, the parser cannot
                    // know which one it was
                    if oper == pt::UserDefinedOperator::Subtract ||
                        oper == pt::UserDefinedOperator::Negate
                    {
                        oper = match func.params.len() {
                            1 => pt::UserDefinedOperator::Negate,
                            2 => pt::UserDefinedOperator::Subtract,
                            _ => {
                                diagnostics.push(
                                    Diagnostic::builder(using_function.loc, Level::Error)
                                        .message("user defined operator function for '-' must have 1 parameter for negate, or 2 parameters for subtract")
                                        .note(loc, format!("definition of '{function_name}'"))
                                        .build(),
                                );
                                continue;
                            }
                        }
                    };

                    if func.params.len() != oper.args() ||
                        func.params.iter().any(|param| param.ty != *ty)
                    {
                        diagnostics.push(
                            Diagnostic::builder( using_function.loc, Level::Error)
                                .message(format!(
                                    "user defined operator function for '{}' must have {} arguments of type {}",
                                    oper, oper.args(), ty.to_string(self.ctx)
                                ),)
                                .note(loc, format!("definition of '{function_name}'"))
                                .build(),
                        );

                        continue;
                    }

                    if oper.is_comparison() {
                        if func.returns.len() != 1 || func.returns[0].ty != Type::Bool {
                            diagnostics.push(
                                Diagnostic::builder( using_function.loc, Level::Error)
                                    .message(format!(
                                        "user defined operator function for '{oper}' must have one bool return type",
                                    ))
                                    .note( loc, format!("definition of '{function_name}'"))
                                    .build(),
                            );

                            continue;
                        }
                    } else if func.returns.len() != 1 || func.returns[0].ty != *ty {
                        diagnostics.push(
                            Diagnostic::builder( using_function.loc, Level::Error)
                                .message(format!(
                                    "user defined operator function for '{}' must have single return type {}",
                                    oper, ty.to_string(self.ctx)
                                ))
                                .note( loc, format!("definition of '{function_name}'"))
                                .build(),
                        );

                        continue;
                    }

                    if !matches!(func.mutability, Mutability::Pure(_)) {
                        diagnostics.push(
                            Diagnostic::builder( using_function.loc, Level::Error)
                                .message(format!(
                                    "user defined operator function for '{oper}' must have pure mutability",
                                ))
                                .note( loc, format!("definition of '{function_name}'"))
                                .build(),
                        );

                        continue;
                    }

                    if let Some(existing) = user_defined_operator_binding(ty, oper, self.ctx) {
                        if existing.function_no != func_no {
                            diagnostics.push(
                                Diagnostic::builder(using_function.loc, Level::Error)
                                    .message(format!(
                                        "user defined operator for '{oper}' redefined"
                                    ))
                                    .note(
                                        existing.loc,
                                        format!(
                                            "previous definition of '{oper}' was '{}'",
                                            self.ctx.functions[existing.function_no].id
                                        ),
                                    )
                                    .build(),
                            );
                        } else {
                            diagnostics.push(
                                Diagnostic::builder(using_function.loc, Level::Warning)
                                    .message( format!(
                                        "user defined operator for '{oper}' redefined to same function"
                                    ))
                                    .note(
                                        existing.loc,
                                        format!(
                                            "previous definition of '{oper}' was '{}'",
                                            self.ctx.functions[existing.function_no].id
                                        )
                                    )
                                    .build(),
                            );
                        }
                        continue;
                    }

                    Some(oper)
                } else {
                    if let Some(ty) = &ty {
                        let dummy = Expression::Variable { loc, ty: ty.clone(), var_no: 0 };

                        if dummy
                            .cast(
                                &loc,
                                &func.params[0].ty,
                                true,
                                self.ctx,
                                &mut Diagnostics::default(),
                            )
                            .is_err()
                        {
                            diagnostics.push(
                                Diagnostic::builder(function_name.loc, Level::Error)
                                    .message(format!("function cannot be used since first argument is '{}' rather than the required '{}'", func.params[0].ty.to_string(self.ctx), ty.to_string(self.ctx)),)
                                    .note(
                                        loc,
                                        format!("definition of '{function_name}'"),
                                    )
                                    .build(),
                            );

                            continue;
                        }
                    }

                    None
                };

                res.push(UsingFunction { loc: using_function.loc, function_no: func_no, oper });
            }
        }

        Ok(UsingList::Functions(res))
    }

    fn resolve_global(
        &mut self,
        global: &pt::Identifier,
        ty: &Option<Type>,
        file_no: &mut Option<usize>,
    ) {
        if global.name == "global" {
            if self.contract_no.is_some() {
                self.ctx.diagnostics.push(Diagnostic::error(
                    global.loc,
                    format!("'{}' on using within contract not permitted", global.name),
                ));
            } else {
                match &ty {
                    Some(Type::Struct(_)) | Some(Type::UserType(_)) | Some(Type::Enum(_)) => {
                        *file_no = None;
                    }
                    _ => {
                        self.ctx.diagnostics.push(Diagnostic::error(
                            global.loc,
                            format!("'{}' only permitted on user defined types", global.name),
                        ));
                    }
                }
            }
        } else {
            self.ctx.diagnostics.push(Diagnostic::error(
                global.loc,
                format!("'{}' not expected, did you mean 'global'?", global.name),
            ));
        }
    }

    pub fn finish(&mut self) -> Option<Using> {
        self.using.take()
    }
}

/// Internal error type for using resolution logic
#[derive(Debug, Error)]
pub enum UsingResolverError {
    #[error("Noops!")]
    Noops,
}

impl<'a> SemanticVisitor for UsingResolver<'a> {
    fn visit_sema_source_unit_part(
        &mut self,
        part: &mut super::ast::SourceUnitPart,
    ) -> Result<(), Self::Error> {
        if let pt::SourceUnitPart::Using(_) = part.part {
            self.ctx.reject(&part.annotations, "using");
            part.visit(self)?;

            if let Some(using) = self.finish() {
                self.ctx.using.push(using);
            }
        }
        Ok(())
    }
}

impl<'a> Visitor for UsingResolver<'a> {
    type Error = UsingResolverError;

    fn visit_using(&mut self, using: &mut pt::Using) -> Result<(), Self::Error> {
        let mut diagnostics = Diagnostics::default();

        if let Some(contract_no) = self.contract_no {
            if self.ctx.contracts[contract_no].is_interface() {
                self.ctx
                    .diagnostics
                    .push(Diagnostic::error(using.loc, "using for not permitted in interface"));
                return Ok(());
            }
        }

        let ty = if let Some(expr) = &using.ty {
            match self.ctx.resolve_type(
                self.no,
                self.contract_no,
                ResolveTypeContext::None,
                expr,
                &mut diagnostics,
            ) {
                Ok(Type::Contract(contract_no)) if self.ctx.contracts[contract_no].is_library() => {
                    self.ctx.diagnostics.push(Diagnostic::error(
                        expr.loc(),
                        format!("using for library '{expr}' type not permitted"),
                    ));
                    return Ok(());
                }
                Ok(ty) => Some(ty),
                Err(_) => {
                    self.ctx.diagnostics.extend(diagnostics);
                    return Ok(());
                }
            }
        } else {
            if self.contract_no.is_none() {
                self.ctx.diagnostics.push(Diagnostic::error(
                    using.loc,
                    "using must be bound to specific type, '*' cannot be used on file scope"
                        .to_string(),
                ));
                return Ok(());
            }
            None
        };

        let list = match &using.list {
            pt::UsingList::Library(library) => self.resolve_library(library, &mut diagnostics)?,
            pt::UsingList::Functions(functions) => {
                self.resove_functions(using, functions, &ty, &mut diagnostics)?
            }
            pt::UsingList::Error => unimplemented!(),
        };

        let mut file_no = Some(self.no);

        if let Some(global) = &using.global {
            self.resolve_global(global, &ty, &mut file_no)
        }

        self.ctx.diagnostics.extend(diagnostics);
        self.using.replace(Using { list, ty, file_no });

        Ok(())
    }
}

/// Given the type and oper, find the user defined operator function binding.
/// Note there can only be one.
pub(crate) fn user_defined_operator_binding<'a>(
    ty: &Type,
    oper: pt::UserDefinedOperator,
    ctx: &'a Context,
) -> Option<&'a UsingFunction> {
    let oper = Some(oper);

    ctx.using.iter().filter(|using| Some(ty) == using.ty.as_ref()).find_map(|using| {
        if let UsingList::Functions(funcs) = &using.list {
            funcs.iter().find(|using| using.oper == oper)
        } else {
            None
        }
    })
}

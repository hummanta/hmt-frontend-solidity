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

use std::{
    fs,
    io::{BufWriter, Write as _},
    path::Path,
};

use cranelift::{
    codegen::settings,
    module::{default_libcall_names, Module},
    object::{ObjectBuilder, ObjectModule},
    prelude::{isa, FunctionBuilder, FunctionBuilderContext},
};
use target_lexicon::Triple;

use crate::{
    ast::Program,
    emit::{CraneliftEmitter, EmitContext},
};

pub struct Codegen {
    module: ObjectModule,
    ir: String,
}

impl Codegen {
    pub fn new() -> Self {
        let flag = settings::Flags::new(settings::builder());

        // Target ISA is same as host machine.
        let isa = isa::lookup(Triple::host()).unwrap().finish(flag).unwrap();

        let builder = ObjectBuilder::new(isa, "", default_libcall_names()).unwrap();
        let module = ObjectModule::new(builder);

        Self { module, ir: String::new() }
    }

    pub fn gen(&mut self, program: &Program) {
        let mut module_ctx = self.module.make_context();
        let mut builder_ctx = FunctionBuilderContext::new();
        let builder = FunctionBuilder::new(&mut module_ctx.func, &mut builder_ctx);

        let mut ctx = EmitContext::new(&mut self.module, builder);
        let mut emitter = CraneliftEmitter::new(&mut ctx);
        program.accept(&mut emitter);

        self.ir.push_str(&format!("{}\n", module_ctx.func));
    }

    pub fn write(&self, path: &Path) {
        let file = fs::File::create(path).unwrap();
        let mut buf_writer = BufWriter::new(file);
        buf_writer.write_all(self.ir.as_bytes()).unwrap();
    }
}

impl Default for Codegen {
    fn default() -> Self {
        Self::new()
    }
}

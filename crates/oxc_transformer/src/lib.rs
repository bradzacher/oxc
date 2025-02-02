#![allow(clippy::wildcard_imports)]

//! Transformer / Transpiler
//!
//! References:
//! * <https://www.typescriptlang.org/tsconfig#target>
//! * <https://babel.dev/docs/presets>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformer.ts>

// Core
mod compiler_assumptions;
mod context;
// Presets: <https://babel.dev/docs/presets>
mod decorators;
mod react;
mod typescript;

use std::rc::Rc;

use oxc_allocator::Allocator;
use oxc_ast::{
    ast::*,
    visit::{walk_mut, VisitMut},
};
use oxc_diagnostics::Error;
use oxc_semantic::Semantic;
use oxc_span::SourceType;

pub use crate::{
    compiler_assumptions::CompilerAssumptions,
    decorators::{Decorators, DecoratorsOptions},
    react::{React, ReactDisplayName, ReactJsx, ReactJsxSelf, ReactJsxSource, ReactOptions},
    typescript::{TypeScript, TypeScriptOptions},
};

use crate::context::{Ctx, TransformCtx};

#[allow(unused)]
#[derive(Debug, Default, Clone)]
pub struct TransformOptions {
    // Core
    /// Set assumptions in order to produce smaller output.
    /// For more information, check the [assumptions](https://babel.dev/docs/assumptions) documentation page.
    pub assumptions: CompilerAssumptions,

    // Plugins
    /// [proposal-decorators](https://babeljs.io/docs/babel-plugin-proposal-decorators)
    pub decorators: DecoratorsOptions,

    /// [preset-typescript](https://babeljs.io/docs/babel-preset-typescript)
    pub typescript: TypeScriptOptions,

    /// [preset-react](https://babeljs.io/docs/babel-preset-react)
    pub react: ReactOptions,
}

#[allow(unused)]
pub struct Transformer<'a> {
    ctx: Ctx<'a>,
    // NOTE: all callbacks must run in order.
    x0_typescript: TypeScript<'a>,
    x1_react: React<'a>,
    x2_decorators: Decorators<'a>,
}

impl<'a> Transformer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_type: SourceType,
        semantic: Semantic<'a>,
        options: TransformOptions,
    ) -> Self {
        let ctx = Rc::new(TransformCtx::new(allocator, source_type, semantic));
        Self {
            ctx: Rc::clone(&ctx),
            x0_typescript: TypeScript::new(options.typescript, &ctx),
            x1_react: React::new(options.react, &ctx),
            x2_decorators: Decorators::new(options.decorators, &ctx),
        }
    }

    /// # Errors
    ///
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(mut self, program: &mut Program<'a>) -> Result<(), Vec<Error>> {
        self.visit_program(program);
        let errors = self.ctx.take_errors();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl<'a> VisitMut<'a> for Transformer<'a> {
    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        self.x0_typescript.transform_statement(stmt);
        self.x2_decorators.transform_statement(stmt);
        walk_mut::walk_statement_mut(self, stmt);
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        self.x1_react.transform_expression(expr);
        walk_mut::walk_expression_mut(self, expr);
    }
}

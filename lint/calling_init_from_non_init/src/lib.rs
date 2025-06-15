// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![feature(rustc_private)]
#![feature(let_chains)]
#![warn(unused_extern_crates)]

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_then;
use rustc_errors::Applicability;
use rustc_hir::def::Res;
use rustc_hir::ExprKind;
use rustc_lint::LateLintPass;
use rustc_span::{sym, Span, Symbol};

dylint_linting::impl_late_lint! {
    /// ### What it does
    ///
    /// Checks for calls to functions marked as `#[init]` from functions not marked as `#[init]`.
    ///
    /// ### Why is this bad?
    ///
    /// Functions marked as `#[init]` should only be called from other `#[init]` functions,
    /// since they might be unmapped from the kernel once the init phase completes.
    ///
    /// ### Known problems
    ///
    /// It does not detect indirect function calls through functions pointers or closures,
    /// since the information if they are marked as `#[init]` or not is lost on conversion.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// #[unsafe(link_section = ".init.text")]
    /// fn foo() {}
    ///
    /// fn bar() {
    ///     foo();
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,no_run
    /// #[unsafe(link_section = ".init.text")]
    /// fn foo() {}
    ///
    /// #[unsafe(link_section = ".init.text")]
    /// fn bar() {
    ///     foo();
    /// }
    /// ```
    pub CALLING_INIT_FROM_NON_INIT,
    Deny,
    "call to a function marked `#[init]` from a non-`#[init]` function",
    CallingInitFromNonInit::default()
}

pub struct CallingInitFromNonInit {
    caller_span: Option<Span>,
    link_section: Symbol,
}

impl Default for CallingInitFromNonInit {
    fn default() -> Self {
        Self {
            caller_span: None,
            link_section: Symbol::intern(".init.text"),
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for CallingInitFromNonInit {
    fn check_fn(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        _: &'tcx rustc_hir::Body<'tcx>,
        span: rustc_span::Span,
        def_id: rustc_span::def_id::LocalDefId,
    ) {
        self.caller_span = {
            if let Some(attr) = cx.tcx.get_attr(def_id, sym::link_section)
                && let Some(link_section) = attr.value_str()
                && link_section == self.link_section
            {
                None
            } else {
                Some(span.shrink_to_lo())
            }
        };
    }

    fn check_expr(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        expr: &'tcx rustc_hir::Expr<'tcx>,
    ) {
        let Some(caller_span) = self.caller_span else {
            return;
        };

        let def_id = match expr.kind {
            ExprKind::Call(target, _) => {
                if let ExprKind::Path(path) = &target.kind
                    && let Res::Def(_, def_id) = cx.typeck_results().qpath_res(path, target.hir_id)
                {
                    def_id
                } else {
                    return;
                }
            }
            ExprKind::MethodCall(_, _, _, _) => {
                match cx.typeck_results().type_dependent_def_id(expr.hir_id) {
                    Some(def_id) => def_id,
                    None => return,
                }
            }
            _ => return,
        };

        if let Some(attr) = cx.tcx.get_attr(def_id, sym::link_section)
            && let Some(link_section) = attr.value_str()
            && link_section == self.link_section
        {
            span_lint_and_then(
                cx,
                CALLING_INIT_FROM_NON_INIT,
                expr.span,
                "calling a function marked as `#[init]` from a non-`#[init]` function",
                |diag| {
                    diag.span_suggestion(
                        caller_span,
                        "consider marking the calling function as `#[init]` too",
                        "#[init]\n",
                        Applicability::MaybeIncorrect,
                    );
                },
            );
        }
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}

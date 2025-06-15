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
use rustc_hir::{ByRef, Expr, ExprKind, PatKind, QPath, StmtKind};
use rustc_span::{sym, Span, Symbol};

dylint_linting::impl_late_lint! {
    /// ### What it does
    ///
    /// Checks for access to statics marked as `#[init]` from functions not marked as `#[init]`.
    ///
    /// ### Why is this bad?
    ///
    /// Statics marked as `#[init]` should only be accessed from `#[init]` functions,
    /// since they might be unmapped from the kernel once the init phase completes.
    ///
    /// ### Example
    ///
    /// ```rust,no_run
    /// #[unsafe(link_section = ".init.rodata")]
    /// static FOO: u32 = 0;
    ///
    /// fn bar() -> u32 {
    ///     FOO + 5
    /// }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust,no_run
    /// #[unsafe(link_section = ".init.rodata")]
    /// static FOO: u32 = 0;
    ///
    /// #[unsafe(link_section = ".init.text")]
    /// fn bar() -> u32 {
    ///     FOO + 5
    /// }
    /// ```
    pub ACCESSING_INIT_FROM_NON_INIT,
    Deny,
    "access of a static marked as `#[init]` from a non-`#[init]` function",
    AccessingInitFromNonInit::default()
}

pub struct AccessingInitFromNonInit {
    caller_span: Option<Span>,
    init_text: Symbol,
    init_rodata: Symbol,
    init_data: Symbol,
}

impl Default for AccessingInitFromNonInit {
    fn default() -> Self {
        Self {
            caller_span: None,
            init_text: Symbol::intern(".init.text"),
            init_rodata: Symbol::intern(".init.rodata"),
            init_data: Symbol::intern(".init.data"),
        }
    }
}

impl<'tcx> rustc_lint::LateLintPass<'tcx> for AccessingInitFromNonInit {
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
                && link_section == self.init_text
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

        match expr.kind {
            ExprKind::AddrOf(_, _, inner) => {
                self.check_init_static(cx, inner, expr.span, caller_span);
            }
            ExprKind::MethodCall(_, receiver, _, _) => {
                self.check_init_static(cx, receiver, expr.span, caller_span);
            }
            _ => {}
        }
    }

    fn check_stmt(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        stmt: &'tcx rustc_hir::Stmt<'tcx>,
    ) {
        if let Some(caller_span) = self.caller_span
            && let StmtKind::Let(loc) = stmt.kind
            && let PatKind::Binding(ba, _, _, _) = loc.pat.kind
            && let ByRef::Yes(_) = ba.0
            && let Some(init) = loc.init
        {
            self.check_init_static(cx, init, init.span, caller_span);
        }
    }
}

impl AccessingInitFromNonInit {
    fn check_init_static(
        &self,
        cx: &rustc_lint::LateContext<'_>,
        mut expr: &Expr<'_>,
        mut error_span: Span,
        caller_span: Span,
    ) {
        if error_span.from_expansion() {
            error_span = expr.span;
        }

        while let ExprKind::Field(e, _) = expr.kind {
            expr = e;
        }

        if let ExprKind::Path(q_path) = expr.kind
            && let QPath::Resolved(_, path) = q_path
            && let Res::Def(_, def_id) = path.res
            && let Some(attr) = cx.tcx.get_attr(def_id, sym::link_section)
            && let Some(link_section) = attr.value_str()
            && (link_section == self.init_rodata || link_section == self.init_data)
        {
            span_lint_and_then(
                cx,
                ACCESSING_INIT_FROM_NON_INIT,
                error_span,
                "accessing a static marked as `#[init]` from a non-`#[init]` function",
                |diag| {
                    diag.span_suggestion(
                        caller_span,
                        "consider marking the accessing function as `#[init]` too",
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

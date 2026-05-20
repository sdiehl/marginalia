use marginalia::{span, Span};
use marginalia_attach::HasSpan;

#[derive(Clone, Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    Let { name: String, value: Expr },
    Print(Expr),
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Num(i64),
    Var(String),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Paren(Box<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl BinOp {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
        }
    }
}

impl Program {
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for Stmt {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for Expr {
    fn span(&self) -> Span {
        self.span
    }
}

#[must_use]
pub fn mk_expr(lo: usize, hi: usize, kind: ExprKind) -> Expr {
    Expr {
        kind,
        span: span(lo, hi),
    }
}

#[must_use]
pub fn mk_stmt(lo: usize, hi: usize, kind: StmtKind) -> Stmt {
    Stmt {
        kind,
        span: span(lo, hi),
    }
}

#[must_use]
pub fn collect_spans(program: &Program) -> Vec<Span> {
    let mut out = vec![program.span];
    for stmt in &program.stmts {
        out.push(stmt.span);
        match &stmt.kind {
            StmtKind::Let { value, .. } | StmtKind::Print(value) => {
                collect_expr_spans(value, &mut out);
            }
        }
    }
    out
}

fn collect_expr_spans(e: &Expr, out: &mut Vec<Span>) {
    out.push(e.span);
    match &e.kind {
        ExprKind::Num(_) | ExprKind::Var(_) => {}
        ExprKind::Bin(_, l, r) => {
            collect_expr_spans(l, out);
            collect_expr_spans(r, out);
        }
        ExprKind::Paren(inner) => collect_expr_spans(inner, out),
    }
}

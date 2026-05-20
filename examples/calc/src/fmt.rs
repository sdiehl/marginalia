use marginalia_pretty::{concat, group, hardline, text, with_trivia, Doc, Format};

use crate::ast::{BinOp, Expr, ExprKind, Program, Stmt, StmtKind};

impl Format for Program {
    fn doc(&self) -> Doc {
        let mut parts: Vec<Doc> = Vec::with_capacity(self.stmts.len() * 2);
        for (i, s) in self.stmts.iter().enumerate() {
            if i > 0 {
                parts.push(hardline());
            }
            parts.push(s.doc());
        }
        concat(parts)
    }
}

impl Format for Stmt {
    fn doc(&self) -> Doc {
        let body = match &self.kind {
            StmtKind::Let { name, value } => concat([
                text("let "),
                text(name.clone()),
                text(" = "),
                group(value.doc()),
                text(";"),
            ]),
            StmtKind::Print(e) => concat([text("print "), group(e.doc()), text(";")]),
        };
        with_trivia(self.span, body)
    }
}

impl Format for Expr {
    fn doc(&self) -> Doc {
        let body = match &self.kind {
            ExprKind::Num(n) => text(n.to_string()),
            ExprKind::Var(s) => text(s.clone()),
            ExprKind::Bin(op, l, r) => bin_doc(*op, l, r),
            ExprKind::Paren(inner) => concat([text("("), inner.doc(), text(")")]),
        };
        with_trivia(self.span, body)
    }
}

fn bin_doc(op: BinOp, lhs: &Expr, rhs: &Expr) -> Doc {
    concat([
        lhs.doc(),
        text(" "),
        text(op.as_str()),
        text(" "),
        rhs.doc(),
    ])
}

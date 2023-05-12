use clap::Parser;
use std::error::Error;

mod b;
#[derive(Parser, Debug)]
struct Args {
    target: u32,

    values: Vec<u32>,
}

fn next(indexes: &mut [usize]) -> bool {
    let p = (1..indexes.len())
        .rev()
        .find(|&i| indexes[i - 1] < indexes[i]);
    if let Some(p) = p {
        // switch indexes[p-1] with the smallest value in indexes[p..] that is greater than indexes[p-1]
        for i in (p..indexes.len()).rev() {
            if indexes[i] > indexes[p - 1] {
                indexes.swap(i, p - 1);
                break;
            }
        }
        // reverse indexes[p..]
        indexes[p..].reverse();
        return true;
    }
    false
}

struct PermIter {
    is: Vec<usize>,
}

impl PermIter {
    fn new(n: usize) -> Self {
        Self {
            is: Vec::with_capacity(n),
        }
    }
}

impl Iterator for PermIter {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is.is_empty() {
            for i in 0..self.is.capacity() {
                self.is.push(i);
            }
            return Some(self.is.clone());
        }

        if next(&mut self.is) {
            return Some(self.is.clone());
        }

        None
    }
}

#[derive(Debug, Clone)]
enum Expr {
    Value(f64),
    Op(Operator, Box<Expr>, Box<Expr>),
}

impl Expr {
    fn eval(&self) -> f64 {
        match self {
            Self::Value(v) => *v,
            Self::Op(op, lhs, rhs) => op.apply(lhs.eval(), rhs.eval()),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Value(_) => 0,
            Self::Op(_, lhs, rhs) => 1 + lhs.len().max(rhs.len()),
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(v) => write!(f, "{}", v),
            Self::Op(op, lhs, rhs) => write!(f, "({} {} {})", lhs, op, rhs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_expr_display() {
        let e = Expr::Op(
            Operator::Add,
            Box::new(Expr::Value(1.0)),
            Box::new(Expr::Op(
                Operator::Multiply,
                Box::new(Expr::Value(2.0)),
                Box::new(Expr::Value(3.0)),
            )),
        );
        assert_eq!(format!("{}", e), "(1 + (2 * 3))");
    }

    #[test]
    fn test_len() {
        let e = Expr::Value(1.0);
        assert_eq!(e.len(), 0);

        let e = Expr::Op(
            Operator::Add,
            Box::new(Expr::Value(0.0)),
            Box::new(Expr::Value(0.0)),
        );
        assert_eq!(e.len(), 1);
    }
}

const ALL_OPERATORS: &[Operator] = &[
    Operator::Add,
    Operator::Subtract,
    Operator::Multiply,
    Operator::Divide,
];

#[derive(Debug, Clone, Copy)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Operator {
    fn apply(&self, lhs: f64, rhs: f64) -> f64 {
        match self {
            Self::Add => lhs + rhs,
            Self::Subtract => lhs - rhs,
            Self::Multiply => lhs * rhs,
            Self::Divide => lhs / rhs,
        }
    }

    fn all() -> &'static [Self] {
        ALL_OPERATORS
    }

    fn to_str(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
        }
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

struct Op {
    op: Operator,
    rhs: f64,
    next: Option<Box<Op>>,
}

impl Op {
    fn eval(&self, v: f64) -> f64 {
        let v = self.op.apply(v, self.rhs);
        match self.next {
            Some(ref next) => next.eval(v),
            None => v,
        }
    }

    fn len(&self) -> usize {
        match self.next {
            Some(ref next) => 1 + next.len(),
            None => 1,
        }
    }
}

// fn is_better(a: &Op, da: f64, b: Option<&Op>, db: f64) -> bool {
//     if let Some(b) = b {
//         if (da - db).abs() < 0.0001 {
//             a.len() < b.len()
//         } else {
//             da < db
//         }
//     } else {
//         true
//     }
// }

// fn find_best(values: &[u32], target: f64) -> Option<Op> {
//     let mut best: Option<Op> = None;
//     let mut best_diff = f64::MAX;
//     for idx in PermIter::new(values.len()) {
//         let values = idx.iter().map(|&i| values[i]).collect::<Vec<_>>();
//         let init = values[0] as f64;
//         if let Some(o) = search(init, &values[1..], target as f64) {
//             let v = o.eval(init);
//             let diff = (v - target).abs();
//             if is_better(&o, diff, best.as_ref(), best_diff) {
//                 best = Some(o);
//                 best_diff = diff;
//             }
//         }
//     }
//     best
// }

// fn search(init: f64, values: &[u32], target: f64) -> Option<Op> {
//     let mut best: Option<Op> = None;
//     let mut best_diff = f64::MAX;

//     if let Some(&v) = values.first() {
//         for &op in Operator::all() {
//             let o = Op {
//                 op,
//                 rhs: v as f64,
//                 next: None,
//             };
//             let v = o.eval(init);
//             let d = (v - target).abs();
//             if is_better(&o, d, best.as_ref(), best_diff) {
//                 best = Some(o);
//                 best_diff = d;
//             }

//             if let Some(o) = search(v, &values[1..], target) {
//                 let o = Op {
//                     op,
//                     rhs: v,
//                     next: Some(Box::new(o)),
//                 };
//                 let v = o.eval(init);
//                 let d = (v - target).abs();
//                 if is_better(&o, d, best.as_ref(), best_diff) {
//                     best = Some(o);
//                     best_diff = d;
//                 }
//             }
//         }
//         best
//     } else {
//         None
//     }
// }

// fn op_to_str(op: &Op) -> String {
//     if let Some(ref next) = op.next {
//         format!("{} {} {}", op.op, op.rhs, op_to_str(next.as_ref()))
//     } else {
//         format!("{} {}", op.op, op.rhs)
//     }
// }

// fn is_better2(a: (&Expr, f64), b: Option<&(Expr, f64)>) -> bool {
//     let (a, a_diff) = a;
//     if let Some((b, b_diff)) = b {
//         if (a_diff - b_diff).abs() < 0.0001 {
//             a.len() < b.len()
//         } else {
//             a_diff < *b_diff
//         }
//     } else {
//         true
//     }
// }

// fn search_naive(e: &Expr, values: &[u32], target: f64) -> Option<(Expr, f64)> {
//     let mut best: Option<(Expr, f64)> = Some((e.clone(), (target - e.eval()).abs()));
//     if let Some(&v) = values.first() {
//         let v = v as f64;
//         for op in Operator::all() {
//             let e = Expr::Op(*op, Box::new(e.clone()), Box::new(Expr::Value(v)));
//             let v = e.eval();
//             let d = (v - target).abs();
//             if is_better2((&e, d), best.as_ref()) {
//                 best = Some((e.clone(), d));
//             }

//             if let Some(p) = search_naive(&e, &values[1..], target) {
//                 let (e, d) = p;
//                 if is_better2((&e, d), best.as_ref()) {
//                     best = Some((e, d));
//                 }
//             }
//         }
//         best
//     } else {
//         None
//     }
// }

// fn find_best_naive(values: &[u32], target: f64) -> Option<Expr> {
//     let mut best: Option<(Expr, f64)> = None;
//     for idx in PermIter::new(values.len()) {
//         let values = idx.iter().map(|&i| values[i]).collect::<Vec<_>>();
//         let a = values[0] as f64;
//         let b = values[1] as f64;
//         for &op in Operator::all() {
//             let e = Expr::Op(op, Box::new(Expr::Value(a)), Box::new(Expr::Value(b)));
//         }
//     }
//     None
// }

fn is_better(a: (usize, f64), b: Option<(usize, f64)>) -> bool {
    if let Some((bn, bd)) = b {
        let (an, ad) = a;
        if (ad - bd).abs() < 0.0001 {
            an < bn
        } else {
            ad < bd
        }
    } else {
        true
    }
}

fn search(e: &Expr, values: &[f64], target: f64) -> Option<Expr> {
    let mut best = e.clone();
    let mut best_diff = (target - e.eval()).abs();
    if let Some(v) = values.first() {
        for &op in Operator::all() {
            let e = Expr::Op(op, Box::new(e.clone()), Box::new(Expr::Value(*v)));
            if let Some(e) = search(&e, &values[1..], target) {
                let v = e.eval();
                let diff = (v - target).abs();
                if is_better((e.len(), diff), Some((best.len(), best_diff))) {
                    best = e;
                    best_diff = diff;
                }
            }
        }
        Some(best)
    } else {
        None
    }
}

fn find_best(values: &[f64], target: f64) -> Option<Expr> {
    let mut best: Option<Expr> = None;
    let mut best_diff = f64::MAX;
    for idx in PermIter::new(values.len()) {
        let values = idx.iter().map(|&i| values[i]).collect::<Vec<_>>();
        for &op in Operator::all() {
            let e = Expr::Op(
                op,
                Box::new(Expr::Value(values[0])),
                Box::new(Expr::Value(values[1])),
            );

            if let Some(e) = search(&e, &values[2..], target) {
                let v = e.eval();
                let diff = (v - target).abs();
                if is_better((e.len(), diff), best.as_ref().map(|b| (b.len(), best_diff))) {
                    best = Some(e);
                    best_diff = diff;
                }
            }
        }
    }
    best
}

fn main() -> Result<(), Box<dyn Error>> {
    b::main()?;

    // let mut args = Args::parse();

    // args.values.sort();

    // let values = args.values.iter().map(|&v| v as f64).collect::<Vec<_>>();

    // let best = find_best(&values, args.target as f64).unwrap();
    // println!("{} = {}", best, best.eval());

    Ok(())
}

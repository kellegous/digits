use clap::{Parser, ValueEnum};
use std::io::prelude::*;
use std::{error::Error, io};

#[derive(Debug, Clone)]
struct Puzzle {
    target: u32,
    digits: Vec<u32>,
}

impl Puzzle {
    fn from_arg(s: &str) -> Result<Self, String> {
        s.parse::<Self>().map_err(|e| e.to_string())
    }

    fn all_solutions(&self) -> Vec<Vec<BinOp>> {
        fn collect_solutions(
            digits: &[u32],
            target: u32,
            steps: &[BinOp],
            solutions: &mut Vec<Vec<BinOp>>,
        ) {
            for (a, b) in all_pairs_in(digits) {
                let mut rest = digits.to_vec();
                rest.remove(rest.iter().position(|&v| v == a).unwrap());
                rest.remove(rest.iter().position(|&v| v == b).unwrap());

                for op in BinOp::all_for(a, b) {
                    let result = match op.evaluate() {
                        Some(v) => v,
                        None => continue,
                    };

                    let mut steps = steps.to_vec();
                    steps.push(op);
                    if result == target {
                        solutions.push(steps);
                    } else {
                        rest.push(result);
                        collect_solutions(&rest, target, &steps, solutions);
                        rest.pop();
                    }
                }
            }
        }

        let mut solutions = Vec::new();
        collect_solutions(&self.digits, self.target, &[], &mut solutions);
        solutions
    }
}

impl std::str::FromStr for Puzzle {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ia = s.find('=').ok_or("missing =")?;
        let target = s[ia + 1..].trim().parse::<u32>()?;
        let digits = s[..ia]
            .split(',')
            .map(|s| s.trim().parse::<u32>())
            .collect::<Result<Vec<_>, _>>()?;
        if digits.len() != 6 {
            return Err("expected 6 values".into());
        }

        Ok(Self { target, digits })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Add => "+",
                Self::Sub => "-",
                Self::Mul => "*",
                Self::Div => "/",
            }
        )
    }
}

#[derive(Debug, Clone)]
struct BinOp {
    op: Op,
    l: u32,
    r: u32,
}

impl BinOp {
    fn new(op: Op, l: u32, r: u32) -> Self {
        Self { op, l, r }
    }

    fn evaluate(&self) -> Option<u32> {
        let l = self.l;
        let r = self.r;
        match self.op {
            Op::Add => Some(l + r),
            Op::Sub => {
                if l >= r {
                    Some(l - r)
                } else {
                    None
                }
            }
            Op::Mul => Some(l * r),
            Op::Div => {
                if r != 0 && (l % r) == 0 {
                    Some(l / r)
                } else {
                    None
                }
            }
        }
    }

    fn all_for(a: u32, b: u32) -> Vec<Self> {
        vec![
            BinOp::new(Op::Add, a, b),
            BinOp::new(Op::Mul, a, b),
            BinOp::new(Op::Sub, a, b),
            BinOp::new(Op::Sub, b, a),
            BinOp::new(Op::Div, a, b),
            BinOp::new(Op::Div, b, a),
        ]
    }
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:3} {} {:3}", self.l, self.op, self.r)
    }
}

fn all_pairs_in(values: &[u32]) -> impl Iterator<Item = (u32, u32)> + '_ {
    values
        .iter()
        .enumerate()
        .flat_map(|(i, &v)| values[i + 1..].iter().map(move |&w| (v, w)))
}

#[derive(Debug, ValueEnum, Clone, Copy)]
enum Strategy {
    Shortest,
    Longest,
    MostDivides,
}

impl Strategy {
    fn is_better(&self, a: &[BinOp], b: &[BinOp]) -> bool {
        match self {
            Self::Shortest => a.len() > b.len(),
            Self::Longest => a.len() < b.len(),
            Self::MostDivides => {
                let na = a.iter().filter(|op| op.op == Op::Div).count();
                let nb = b.iter().filter(|op| op.op == Op::Div).count();
                na < nb || (na == nb && a.len() > b.len())
            }
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_parser= Puzzle::from_arg)]
    puzzle: Option<Puzzle>,

    #[arg(long, value_enum, default_value_t = Strategy::Shortest)]
    strategy: Strategy,
}

fn report(solutions: &[Vec<BinOp>], strategy: Strategy) -> Result<(), Box<dyn Error>> {
    let best = solutions
        .iter()
        .fold(None, |best, soln| match best {
            Some(best) => {
                if strategy.is_better(best, soln) {
                    Some(soln)
                } else {
                    Some(best)
                }
            }
            None => Some(soln),
        })
        .ok_or("no solution found")?;

    for step in best {
        println!("{} = {:3}", step, step.evaluate().unwrap());
    }

    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if let Some(puzzle) = args.puzzle {
        report(&puzzle.all_solutions(), args.strategy)?;
    } else {
        let stdin = io::stdin();
        let mut line = String::new();
        loop {
            line.clear();
            print!("> ");
            io::stdout().flush()?;
            if stdin.read_line(&mut line)? == 0 {
                break;
            }

            let puzzle = match line.trim().parse::<Puzzle>() {
                Ok(puzzle) => puzzle,
                Err(e) => {
                    println!("error: {}", e);
                    continue;
                }
            };

            if let Err(e) = report(&puzzle.all_solutions(), args.strategy) {
                println!("error: {}", e);
                continue;
            }
        }
    }
    Ok(())
}

use clap::Parser;
use std::collections::{HashMap, VecDeque};
use std::io::prelude::*;
use std::path::Iter;
use std::{error::Error, io};

#[derive(Debug, Clone)]
struct Puzzle {
    target: u32,
    values: Vec<u32>,
}

impl Puzzle {
    fn from_arg(s: &str) -> Result<Self, String> {
        s.parse::<Self>().map_err(|e| e.to_string())
    }
}

impl std::str::FromStr for Puzzle {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ia = s.find('=').ok_or("missing =")?;
        let target = s[ia + 1..].trim().parse::<u32>()?;
        let values = s[..ia]
            .split(',')
            .map(|s| s.trim().parse::<u32>())
            .collect::<Result<Vec<_>, _>>()?;
        if values.len() != 6 {
            return Err("expected 6 values".into());
        }

        Ok(Self { target, values })
    }
}

#[derive(Debug, Clone, Copy)]
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

    fn apply(&self) -> Option<u32> {
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

struct Solutions {
    target: u32,
    queue: VecDeque<(Vec<BinOp>, Vec<u32>)>,
}

impl Solutions {
    fn from_puzzle(p: &Puzzle) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back((vec![], p.values.clone()));
        Self {
            target: p.target,
            queue,
        }
    }
}

impl Iterator for Solutions {
    type Item = Vec<BinOp>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (ops, digits) = match self.queue.pop_front() {
                Some(v) => v,
                None => return None,
            };

            for (a, b) in all_pairs_in(&digits) {
                let mut rest = digits.to_vec();
                rest.remove(rest.iter().position(|&v| v == a).unwrap());
                rest.remove(rest.iter().position(|&v| v == b).unwrap());
                for op in BinOp::all_for(a, b) {
                    let result = match op.apply() {
                        Some(v) => v,
                        None => continue,
                    };

                    let mut rest = rest.to_vec();
                    rest.push(result);

                    let mut ops = ops.to_vec();
                    ops.push(op);

                    self.queue.push_back((ops, rest));
                }
            }

            if let Some(result) = ops.last().and_then(|op| op.apply()) {
                if result == self.target {
                    return Some(ops);
                }
            }
        }
    }
}

fn all_pairs_in(values: &[u32]) -> impl Iterator<Item = (u32, u32)> + '_ {
    values
        .iter()
        .enumerate()
        .flat_map(|(i, &v)| values[i + 1..].iter().map(move |&w| (v, w)))
}

fn replace_op_in(values: &[u32], op: &BinOp, res: u32) -> Vec<u32> {
    let mut values = values.to_vec();
    let i = values.iter().position(|&v| v == op.l).unwrap();
    let j = values.iter().position(|&v| v == op.r).unwrap();
    values[i] = res;
    values.remove(j);
    values
}

fn collect_solutions(
    values: &[u32],
    target: u32,
    steps: &[BinOp],
    solutions: &mut Vec<Vec<BinOp>>,
) {
    for (a, b) in all_pairs_in(values) {
        let mut rest = values.to_vec();
        rest.remove(rest.iter().position(|&v| v == a).unwrap());
        rest.remove(rest.iter().position(|&v| v == b).unwrap());

        for op in BinOp::all_for(a, b) {
            let result = match op.apply() {
                Some(v) => v,
                None => continue,
            };

            let mut steps = steps.to_vec();
            steps.push(op);

            rest.push(result);
            collect_solutions(&rest, target, &steps, solutions);
            rest.pop();

            if result == target {
                solutions.push(steps);
            }
        }
    }
}

fn solve(values: &[u32], target: u32) -> Vec<BinOp> {
    let mut steps = Vec::new();
    for (a, b) in all_pairs_in(values) {
        for op in BinOp::all_for(a, b) {
            let result = match op.apply() {
                Some(v) => v,
                None => continue,
            };

            if result == target {
                return vec![op];
            }

            let soln = solve(&replace_op_in(values, &op, result), target);
            if !soln.is_empty() && (steps.is_empty() || soln.len() < steps.len()) {
                steps.clear();
                steps.push(op);
                steps.extend(soln);
            }
        }
    }
    steps
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(value_parser= Puzzle::from_arg)]
    puzzle: Option<Puzzle>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if let Some(puzzle) = args.puzzle {
        for step in solve(&puzzle.values, puzzle.target) {
            println!("{} = {:3}", step, step.apply().unwrap());
        }

        println!("------");

        let mut stats = HashMap::new();

        // let mut solutions = Vec::new();
        // collect_solutions(&puzzle.values, puzzle.target, &[], &mut solutions);
        // for solution in solutions {
        //     *stats.entry(solution.len()).or_insert(0) += 1;
        // }

        for solution in Solutions::from_puzzle(&puzzle) {
            *stats.entry(solution.len()).or_insert(0) += 1;
        }

        println!("{:?}", stats);
        // let mut solutions = Vec::new();
        // collect_solutions(&puzzle.values, puzzle.target, &[], &mut solutions);
        // for solution in solutions {
        //     if solution.len() == 5 {
        //         for step in solution {
        //             println!("{} = {:3}", step, step.apply().unwrap());
        //         }
        //         break;
        //     }
        // }

        // for solution in Solutions::from_puzzle(&puzzle) {
        //     if solution.len() == 5 {
        //         for step in solution {
        //             println!("{} = {:3}", step, step.apply().unwrap());
        //         }
        //         break;
        //     }
        // }
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

            for step in solve(&puzzle.values, puzzle.target) {
                println!("{} = {:3}", step, step.apply().unwrap());
            }

            println!("------");
            let mut solutions = Vec::new();
            collect_solutions(&puzzle.values, puzzle.target, &[], &mut solutions);
            for solution in solutions {
                if solution.len() == 5 {
                    for step in solution {
                        println!("{} = {:3}", step, step.apply().unwrap());
                    }
                    break;
                }
            }
        }
    }
    Ok(())
}

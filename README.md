# NYT Digits Solver

A silly little solver for the [NYT Digits puzzle](https://www.nytimes.com/puzzles/digits).

## Building

just like any other rust project.

```
cargo build --release
```

## Usage

Interactive mode allows you to enter each of the 5 daily puzzles.

```
$ digits
> 1,2,3,4,5,25=65
 25 -  4 =   21
  3 *  21 =  63
  2 +  63 =  65
> 2,3,7,9,10,15=141
 10 *  15 = 150
150 -   9 = 141
```

Single shot mode.

```
$ digits 1,2,3,4,5,25=65
 25 -   4 =  21
  3 *  21 =  63
  2 +  63 =  65
```
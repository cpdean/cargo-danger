## `danger`

A utility for exploring a crate's dependencies.


`danger` lets you know which of your projects dependencies relies on `unsafe`
code, giving you a count of lines that fall under `unsafe fn` definitions, or
`unsafe` blocks within a function definition.

### Installation

Currently `danger` can only be compiled from crates.io for installation.  You
will need cargo installed and a rust toolchain.  Fortunately this is a tool for
Rust programmers, so if you want to use `danger` you probably have everything
you need already:

```
cargo install cargo-danger
# 6-8 minutes compiling
```

### Usage

From the root of a cargo project, run `cargo danger`:

```
0 - ❯❯❯ cd rayon
0 - ❯❯❯ cargo danger
nodrop, 4
num_cpus, 11
crossbeam-epoch, 91
crossbeam-deque, 22
crossbeam, 60
rayon, 234
lazy_static, 2
arrayvec, 58
rayon-core, 101
crossbeam-utils, 6
0 - ❯❯❯
```

You can also count the number of unsafe lines in a given directory:

```
0 - ❯❯❯ cargo danger examples
0 unsafe lines
0 - ❯❯❯ cargo danger rayon-core/
101 unsafe lines
0 - ❯❯❯ cargo danger rayon-futures/
16 unsafe lines
0 - ❯❯❯ cargo danger rayon-demo/
17 unsafe lines
0 - ❯❯❯ cargo danger src
100 unsafe lines
0 - ❯❯❯ 
```

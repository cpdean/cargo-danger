use cargo::core::dependency::Kind;
use cargo::core::package_id::PackageId;
use cargo::core::{Package, Workspace};
use cargo::ops;
use cargo::util::CargoResult;
use cargo::Config;

use std::env;
use std::path::Path;
use syn::Item;

use std::fs::File;
use std::io::Read;

use std::collections::HashSet;

use cargo::util::important_paths::find_root_manifest_for_wd;

type Result<T> = std::result::Result<T, Box<std::error::Error>>;

/*
struct CodeLine {
    file_path: String,
    line_number: usize,
    raw_line: String,
}
*/

#[derive(Debug)]
struct UnsafeLines {
    package: Package,
    //lines: Vec<CodeLine>
    lines: usize,
}

fn main() -> Result<()> {
    let argv: Vec<_> = env::args().collect();
    if argv.len() == 1 {
        // by default cargo-danger will warn you of unsafe lines of code in your
        // project's dependencies
        let mut config = Config::default().expect("No idea why this would fail");
        let result = print_files(&mut config);
        dbg!("got some packages");
        match result {
            Ok(packs) => {
                for p in packs {
                    let UnsafeLines { package, lines } = p;
                    let name = package.package_id().name();
                    if lines > 0 {
                        println!("{}, {}", name, lines);
                    }
                }
            }
            Err(err) => {
                // maybe exit_with_error is best but i cannot find the way to get it to work now
                //cargo::exit_with_error(err, &mut *config.shell());
                panic!("dunno {:?}", err);
            }
        }
    } else {
        // if you want to inspect a specific directory
        println!(
            "{} unsafe lines",
            count_of_unsafe(Path::new(&argv[1]), true)?
        );
    }
    Ok(())
}

fn print_files(config: &mut Config) -> Result<Vec<UnsafeLines>> {
    let root = resolve_roots(config)?;
    let mut packages = vec![];
    // TODO: be able to choose first order vs all deps
    let _packages = resolve_packages(config, root)?;
    let mut open_files = true;
    for p in _packages {
        let things = count_of_unsafe(&p.root(), open_files)?;
        open_files = true;
        packages.push(UnsafeLines {
            package: p,
            lines: things,
        });
    }
    Ok(packages)
}

fn count_if_in(in_unsafe_block: bool) -> usize {
    if in_unsafe_block {
        1
    } else {
        0
    }
}

fn unsafe_things_of_block(block: &syn::Block, in_unsafe_block: bool) -> usize {
    let mut total = 0;
    for s in &block.stmts {
        total += unsafe_things_of_statement(&s, in_unsafe_block);
    }
    total
}

fn unsafe_things_of_expression(expr: &syn::Expr, in_unsafe_block: bool) -> usize {
    use syn::Expr;
    let result: usize = match expr {
        Expr::Box(boxed_expr) => unsafe_things_of_expression(&boxed_expr.expr, in_unsafe_block),
        Expr::InPlace(in_place_expr) => {
            let left = unsafe_things_of_expression(&in_place_expr.place, in_unsafe_block);
            let right = unsafe_things_of_expression(&in_place_expr.value, in_unsafe_block);
            left + right
        }
        Expr::Array(array_exp) => {
            // TODO: i'm not sure how to accumulate individual potential expressions
            // in here so i'm just going to count an array expression as one.
            count_if_in(in_unsafe_block)
        }
        Expr::Call(expr) => {
            /*
            /// A function call expression: `invoke(a, b)`.
            ///
            /// *This type is available if Syn is built with the `"derive"` or
            /// `"full"` feature.*
            pub Call(ExprCall {
                pub attrs: Vec<Attribute>,
                pub func: Box<Expr>,
                pub paren_token: token::Paren,
                pub args: Punctuated<Expr, Token![,]>,
            }),
            */
            count_if_in(in_unsafe_block)
        }
        Expr::MethodCall(expr) => {
            /*
            /// A method call expression: `x.foo::<T>(a, b)`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub MethodCall(ExprMethodCall #full {
            pub attrs: Vec<Attribute>,
            pub receiver: Box<Expr>,
            pub dot_token: Token![.],
            pub method: Ident,
            pub turbofish: Option<MethodTurbofish>,
            pub paren_token: token::Paren,
            pub args: Punctuated<Expr, Token![,]>,
            }),
            */
            count_if_in(in_unsafe_block)
        }
        Expr::Tuple(expr) => {
            /*
            /// A tuple expression: `(a, b, c, d)`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Tuple(ExprTuple #full {
                pub attrs: Vec<Attribute>,
                pub paren_token: token::Paren,
                pub elems: Punctuated<Expr, Token![,]>,
            }),
            */
            count_if_in(in_unsafe_block)
        }
        Expr::Binary(expr) => {
            /*
            /// A binary operation: `a + b`, `a * b`.
            ///
            /// *This type is available if Syn is built with the `"derive"` or
            /// `"full"` feature.*
            pub Binary(ExprBinary {
                pub attrs: Vec<Attribute>,
                pub left: Box<Expr>,
                pub op: BinOp,
                pub right: Box<Expr>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Unary(expr) => {
            /*
            /// A unary operation: `!x`, `*x`.
            ///
            /// *This type is available if Syn is built with the `"derive"` or
            /// `"full"` feature.*
            pub Unary(ExprUnary {
                pub attrs: Vec<Attribute>,
                pub op: UnOp,
                pub expr: Box<Expr>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Lit(expr) => {
            /*
            /// A literal in place of an expression: `1`, `"foo"`.
            ///
            /// *This type is available if Syn is built with the `"derive"` or
            /// `"full"` feature.*
            pub Lit(ExprLit {
                pub attrs: Vec<Attribute>,
                pub lit: Lit,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Cast(expr) => {
            /*
            /// A cast expression: `foo as f64`.
            ///
            /// *This type is available if Syn is built with the `"derive"` or
            /// `"full"` feature.*
            pub Cast(ExprCast {
                pub attrs: Vec<Attribute>,
                pub expr: Box<Expr>,
                pub as_token: Token![as],
                pub ty: Box<Type>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Type(expr) => {
            /*
            /// A type ascription expression: `foo: f64`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Type(ExprType #full {
                pub attrs: Vec<Attribute>,
                pub expr: Box<Expr>,
                pub colon_token: Token![:],
                pub ty: Box<Type>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Let(expr) => {
            /*
            /// A `let` guard: `let Some(x) = opt`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Let(ExprLet #full {
                pub attrs: Vec<Attribute>,
                pub let_token: Token![let],
                pub pats: Punctuated<Pat, Token![|]>,
                pub eq_token: Token![=],
                pub expr: Box<Expr>,
            }),
            */
            count_if_in(in_unsafe_block)
        }
        Expr::If(expr) => {
            let cond_result = unsafe_things_of_expression(&expr.cond, in_unsafe_block);
            let then_branch = unsafe_things_of_block(&expr.then_branch, in_unsafe_block);
            match &expr.else_branch {
                Some((_, expr)) => {
                    let else_result = unsafe_things_of_expression(&expr, in_unsafe_block);
                    cond_result + then_branch + else_result
                }
                None => cond_result + then_branch,
            }
        }
        Expr::While(expr) => {
            /*
            /// A while loop: `while expr { ... }`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub While(ExprWhile #full {
                pub attrs: Vec<Attribute>,
                pub label: Option<Label>,
                pub while_token: Token![while],
                pub cond: Box<Expr>,
                pub body: Block,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::ForLoop(expr) => {
            /*
            /// A for loop: `for pat in expr { ... }`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub ForLoop(ExprForLoop #full {
                pub attrs: Vec<Attribute>,
                pub label: Option<Label>,
                pub for_token: Token![for],
                pub pat: Box<Pat>,
                pub in_token: Token![in],
                pub expr: Box<Expr>,
                pub body: Block,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Loop(expr) => {
            /*
            /// Conditionless loop: `loop { ... }`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Loop(ExprLoop #full {
                pub attrs: Vec<Attribute>,
                pub label: Option<Label>,
                pub loop_token: Token![loop],
                pub body: Block,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Match(expr) => {
            /*
            /// A `match` expression: `match n { Some(n) => {}, None => {} }`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Match(ExprMatch #full {
                pub attrs: Vec<Attribute>,
                pub match_token: Token![match],
                pub expr: Box<Expr>,
                pub brace_token: token::Brace,
                pub arms: Vec<Arm>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Closure(expr) => {
            /*
            /// A closure expression: `|a, b| a + b`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Closure(ExprClosure #full {
                pub attrs: Vec<Attribute>,
                pub asyncness: Option<Token![async]>,
                pub movability: Option<Token![static]>,
                pub capture: Option<Token![move]>,
                pub or1_token: Token![|],
                pub inputs: Punctuated<FnArg, Token![,]>,
                pub or2_token: Token![|],
                pub output: ReturnType,
                pub body: Box<Expr>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Unsafe(expr) => {
            /*
            /// An unsafe block: `unsafe { ... }`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Unsafe(ExprUnsafe #full {
                pub attrs: Vec<Attribute>,
                pub unsafe_token: Token![unsafe],
                pub block: Block,
            }),
            */
            let mut total = 0;
            for s in &expr.block.stmts {
                total += unsafe_things_of_statement(&s, true);
            }
            total
        }

        Expr::Block(expr) => {
            /*
            /// A blocked scope: `{ ... }`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Block(ExprBlock #full {
                pub attrs: Vec<Attribute>,
                pub label: Option<Label>,
                pub block: Block,
            }),
            */
            let mut total = 0;
            for s in &expr.block.stmts {
                total += unsafe_things_of_statement(&s, in_unsafe_block);
            }
            total
        }

        Expr::Assign(expr) => {
            /*
            /// An assignment expression: `a = compute()`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Assign(ExprAssign #full {
                pub attrs: Vec<Attribute>,
                pub left: Box<Expr>,
                pub eq_token: Token![=],
                pub right: Box<Expr>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::AssignOp(expr) => {
            /*
            /// A compound assignment expression: `counter += 1`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub AssignOp(ExprAssignOp #full {
                pub attrs: Vec<Attribute>,
                pub left: Box<Expr>,
                pub op: BinOp,
                pub right: Box<Expr>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Field(expr) => {
            /*
            /// Access of a named struct field (`obj.k`) or unnamed tuple struct
            /// field (`obj.0`).
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Field(ExprField {
                pub attrs: Vec<Attribute>,
                pub base: Box<Expr>,
                pub dot_token: Token![.],
                pub member: Member,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Index(expr) => {
            /*
            /// A square bracketed indexing expression: `vector[2]`.
            ///
            /// *This type is available if Syn is built with the `"derive"` or
            /// `"full"` feature.*
            pub Index(ExprIndex {
                pub attrs: Vec<Attribute>,
                pub expr: Box<Expr>,
                pub bracket_token: token::Bracket,
                pub index: Box<Expr>,
            }),

            */
            count_if_in(in_unsafe_block)
        }

        Expr::Range(expr) => {
            /*
            /// A range expression: `1..2`, `1..`, `..2`, `1..=2`, `..=2`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Range(ExprRange #full {
                pub attrs: Vec<Attribute>,
                pub from: Option<Box<Expr>>,
                pub limits: RangeLimits,
                pub to: Option<Box<Expr>>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Path(expr) => {
            /*
            /// A path like `std::mem::replace` possibly containing generic
            /// parameters and a qualified self-type.
            ///
            /// A plain identifier like `x` is a path of length 1.
            ///
            /// *This type is available if Syn is built with the `"derive"` or
            /// `"full"` feature.*
            pub Path(ExprPath {
                pub attrs: Vec<Attribute>,
                pub qself: Option<QSelf>,
                pub path: Path,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Reference(expr) => {
            /*
            /// A referencing operation: `&a` or `&mut a`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Reference(ExprReference #full {
                pub attrs: Vec<Attribute>,
                pub and_token: Token![&],
                pub mutability: Option<Token![mut]>,
                pub expr: Box<Expr>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Break(expr) => {
            /*
            /// A `break`, with an optional label to break and an optional
            /// expression.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Break(ExprBreak #full {
                pub attrs: Vec<Attribute>,
                pub break_token: Token![break],
                pub label: Option<Lifetime>,
                pub expr: Option<Box<Expr>>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Continue(expr) => {
            /*
            /// A `continue`, with an optional label.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Continue(ExprContinue #full {
                pub attrs: Vec<Attribute>,
                pub continue_token: Token![continue],
                pub label: Option<Lifetime>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Return(expr) => {
            /*
            /// A `return`, with an optional value to be returned.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Return(ExprReturn #full {
                pub attrs: Vec<Attribute>,
                pub return_token: Token![return],
                pub expr: Option<Box<Expr>>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Macro(expr) => {
            /*
            /// A macro invocation expression: `format!("{}", q)`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Macro(ExprMacro #full {
                pub attrs: Vec<Attribute>,
                pub mac: Macro,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Struct(expr) => {
            /*
            /// A struct literal expression: `Point { x: 1, y: 1 }`.
            ///
            /// The `rest` provides the value of the remaining fields as in `S { a:
            /// 1, b: 1, ..rest }`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Struct(ExprStruct #full {
                pub attrs: Vec<Attribute>,
                pub path: Path,
                pub brace_token: token::Brace,
                pub fields: Punctuated<FieldValue, Token![,]>,
                pub dot2_token: Option<Token![..]>,
                pub rest: Option<Box<Expr>>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Repeat(expr) => {
            /*
            /// An array literal constructed from one repeated element: `[0u8; N]`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Repeat(ExprRepeat #full {
                pub attrs: Vec<Attribute>,
                pub bracket_token: token::Bracket,
                pub expr: Box<Expr>,
                pub semi_token: Token![;],
                pub len: Box<Expr>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Paren(expr) => {
            /*
            /// A parenthesized expression: `(a + b)`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Paren(ExprParen {
                pub attrs: Vec<Attribute>,
                pub paren_token: token::Paren,
                pub expr: Box<Expr>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Group(expr) => {
            /*
            /// An expression contained within invisible delimiters.
            ///
            /// This variant is important for faithfully representing the precedence
            /// of expressions and is related to `None`-delimited spans in a
            /// `TokenStream`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Group(ExprGroup #full {
                pub attrs: Vec<Attribute>,
                pub group_token: token::Group,
                pub expr: Box<Expr>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Try(expr) => {
            /*
            /// A try-expression: `expr?`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Try(ExprTry #full {
                pub attrs: Vec<Attribute>,
                pub expr: Box<Expr>,
                pub question_token: Token![?],
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Async(expr) => {
            /*
            /// An async block: `async { ... }`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Async(ExprAsync #full {
                pub attrs: Vec<Attribute>,
                pub async_token: Token![async],
                pub capture: Option<Token![move]>,
                pub block: Block,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::TryBlock(expr) => {
            /*
            /// A try block: `try { ... }`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub TryBlock(ExprTryBlock #full {
                pub attrs: Vec<Attribute>,
                pub try_token: Token![try],
                pub block: Block,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Yield(expr) => {
            /*
            /// A yield expression: `yield expr`.
            ///
            /// *This type is available if Syn is built with the `"full"` feature.*
            pub Yield(ExprYield #full {
                pub attrs: Vec<Attribute>,
                pub yield_token: Token![yield],
                pub expr: Option<Box<Expr>>,
            }),
            */
            count_if_in(in_unsafe_block)
        }

        Expr::Verbatim(expr) => {
            /*
            /// Tokens in expression position not interpreted by Syn.
            ///
            /// *This type is available if Syn is built with the `"derive"` or
            /// `"full"` feature.*
            pub Verbatim(ExprVerbatim #manual_extra_traits {
                pub tts: TokenStream,
            }),
            */
            count_if_in(in_unsafe_block)
        }
    };
    result
}

fn unsafe_things_of_statement(item: &syn::Stmt, in_unsafe_block: bool) -> usize {
    match item {
        syn::Stmt::Local(local) => count_if_in(in_unsafe_block),
        syn::Stmt::Expr(_expr_statement) => {
            unsafe_things_of_expression(&_expr_statement, in_unsafe_block)
        }
        syn::Stmt::Semi(_expr_statement, _) => {
            unsafe_things_of_expression(&_expr_statement, in_unsafe_block)
        }
        syn::Stmt::Item(item_statement) => unsafe_things_of_item(&item_statement, in_unsafe_block),
    }
}

fn unsafe_things_of_implitem(item: &syn::ImplItem, in_unsafe_block: bool) -> usize {
    match item {
        // figure it out:  https://docs.rs/syn/0.15.26/syn/enum.ImplItem.html
        syn::ImplItem::Const(_) => 0,
        syn::ImplItem::Method(method_impl) => {
            let mut total = 0;
            for s in &method_impl.block.stmts {
                total += unsafe_things_of_statement(&s, in_unsafe_block);
            }
            total
        }
        syn::ImplItem::Type(_) => {
            // are there unsafe types?
            0
        }
        syn::ImplItem::Existential(_) => 0,
        syn::ImplItem::Macro(_) => 0,
        syn::ImplItem::Verbatim(_) => 0,
    }
}

/// recursively count unsafe things, since items can have more items in them
fn unsafe_things_of_item(item: &Item, in_unsafe_block: bool) -> usize {
    let result: usize = match item {
        Item::Fn(fn_def) => match fn_def.unsafety {
            Some(_) => {
                let mut total = 0;
                for s in &fn_def.block.stmts {
                    total += unsafe_things_of_statement(&s, true);
                }
                total
            }
            None => {
                let mut total = 0;
                for s in &fn_def.block.stmts {
                    total += unsafe_things_of_statement(&s, in_unsafe_block);
                }
                total
            }
        },
        Item::Mod(mod_def) => {
            // TODO: i don't understand the mod_def thing
            match &mod_def.content {
                Some((_brace, items)) => {
                    let mut total = 0;
                    for i in items {
                        total += unsafe_things_of_item(&i, in_unsafe_block);
                    }
                    total
                }
                None => 0,
            }
        }
        Item::Impl(impl_def) => match impl_def.unsafety {
            Some(_) => {
                let mut total = 0;
                for i in &impl_def.items {
                    total += unsafe_things_of_implitem(&i, true);
                }
                total
            }
            None => {
                let mut total = 0;
                for i in &impl_def.items {
                    total += unsafe_things_of_implitem(&i, in_unsafe_block);
                }
                total
            }
        },
        _ => {
            // i don't think other types can hold unsafe code
            // double check whatever is current with the installed version of syn
            // https://docs.rs/syn/0.15.26/syn/enum.Item.html
            0
        }
    };
    result
}

fn unsafe_lines_of_file(file: &syn::File) -> usize {
    let mut unsafes = 0;
    for i in &file.items {
        unsafes += unsafe_things_of_item(&i, false);
    }
    unsafes
}

fn count_of_unsafe(root_dir: &Path, open_files: bool) -> Result<usize> {
    let mut unsafe_total = 0;
    for f in files_of(root_dir)? {
        if let Some(ext) = f.extension() {
            if ext == "rs" {
                if open_files {
                    //let mut file = File::open(dbg!(f))?;
                    let mut file = File::open(f)?;
                    let mut content = String::new();
                    file.read_to_string(&mut content)?;
                    let ast = syn::parse_file(&content)?;
                    let unsafe_lines = unsafe_lines_of_file(&ast);
                    unsafe_total += unsafe_lines;
                }
            }
        }
    }
    // start with just counting files
    Ok(unsafe_total)
}

fn files_of(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                for sub_path in files_of(&path)? {
                    files.push(sub_path);
                }
            } else {
                files.push(entry.path());
            }
        }
    }
    Ok(files)
}

pub fn resolve_roots(config: &Config) -> CargoResult<Package> {
    let root_manifest = find_root_manifest_for_wd(config.cwd())?;
    let workspace = Workspace::new(&root_manifest, config)?;

    Ok(workspace.current()?.clone())
}

pub fn resolve_packages(config: &Config, root_package: Package) -> CargoResult<Vec<Package>> {
    let root_manifest = find_root_manifest_for_wd(config.cwd())?;
    let workspace = Workspace::new(&root_manifest, config)?;

    let (packages, resolve) = ops::resolve_ws(&workspace)?;

    let mut result = HashSet::new();
    let id = root_package.package_id();
    let mut to_check: Vec<&PackageId> = vec![&id];
    while let Some(id) = to_check.pop() {
        if let Ok(package) = packages.get_one(id) {
            if result.insert(package) {
                let deps = resolve.deps_not_replaced(id);
                for dep_id in deps {
                    let dep = package.dependencies().iter()
                        .find(|d| d.matches_id(dep_id))
                        .unwrap_or_else(|| panic!("Looking up a packages dependency in the package failed, failed to find '{}' in '{}'", dep_id, id));
                    if let Kind::Normal = dep.kind() {
                        let dep_id = resolve.replacement(dep_id).unwrap_or(dep_id);
                        to_check.push(dep_id);
                    }
                }
            }
        }
    }

    Ok(result.into_iter().cloned().collect())
}

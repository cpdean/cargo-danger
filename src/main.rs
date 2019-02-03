use cargo::core::dependency::Kind;
use cargo::core::package_id::PackageId;
use cargo::core::{Package, Workspace};
use cargo::ops;
use cargo::util::CargoResult;
use cargo::Config;

use std::collections::HashSet;

use cargo::util::important_paths::find_root_manifest_for_wd;

type Result<T> = std::result::Result<T, Box<std::error::Error>>;

struct CodeLine {
    file_path: String,
    line_number: usize,
    raw_line: String,
}

#[derive(Debug)]
struct UnsafeLines {
    package: Package,
    //lines: Vec<CodeLine>
    lines: usize,
}

fn main() {
    let mut config = Config::default().expect("No idea why this would fail");
    let result = print_files(&mut config);
    dbg!("got some packages");
    match result {
        Ok(packs) => {
            for p in packs {
                let UnsafeLines { package, lines } = p;
                let name = package.package_id().name();
                println!("{}, {}", name, lines);
            }
        }
        Err(err) => {
            // maybe exit_with_error is best but i cannot find the way to get it to work now
            //cargo::exit_with_error(err, &mut *config.shell());
            panic!("dunno {:?}", err);
        }
    }
}

fn print_files(config: &mut Config) -> Result<Vec<UnsafeLines>> {
    let root = resolve_roots(config)?;
    let mut packages = vec![];
    let _packages = resolve_packages(config, root)?;
    for p in _packages {
        let things = count_of_unsafe(&p)?;
        packages.push(UnsafeLines {
            package: p,
            lines: things,
        });
    }
    Ok(packages)
}

fn count_of_unsafe(package: &Package) -> Result<usize> {
    let mut unsafe_total = 0;
    for _f in files_of(package.root())? {
        unsafe_total += 1;
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

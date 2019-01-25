use cargo::core::dependency::Kind;
use cargo::core::{Package, Workspace};
use cargo::core::package_id::PackageId;
use cargo::ops;
use cargo::util::{CargoResult};
use cargo::{CliResult, Config};

use std::collections::HashSet;

use cargo::util::important_paths::find_root_manifest_for_wd;



fn main() {
    let mut config = Config::default().expect("No idea why this would fail");
    let result = print_files(&mut config);
    if let Err(err) = result {
        cargo::exit_with_error(err, &mut *config.shell());
    }
}

fn print_files(config: &mut Config) -> CliResult {
    let root = resolve_roots(config)?;
    let mut packages: Vec<Package> = vec![];
    let _packages = resolve_packages(config, root)?;
    dbg!(_packages.len());
    for p in _packages {
        packages.push(p);
    }
    for p in packages.iter().map(|e| e.root()) {
        println!("{:?}", p);
    }

    Ok(())
}


pub fn resolve_roots(config: &Config) -> CargoResult<Package> {
    let root_manifest = find_root_manifest_for_wd(config.cwd())?;
    let workspace = Workspace::new(&root_manifest, config)?;

    Ok(workspace.current()?.clone())
}

pub fn resolve_packages(
        config: &Config,
        root_package: Package) -> CargoResult<Vec<Package>> {
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

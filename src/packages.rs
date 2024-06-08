use std::collections::HashSet;
use alpm::{Alpm, PackageReason};
use crate::Item;

pub struct ArchPackages {
    alpm: Alpm,
}

impl ArchPackages {
    pub fn new() -> crate::Result<Self> {
        let config = alpm_utils::config::Config::new()?;
        let alpm = alpm_utils::alpm_with_conf(&config)?;
        Ok(Self { alpm })
    }
    pub fn get_root_packages<'a>(&'a mut self) -> impl Iterator<Item=Item<'a>> {
        self.alpm
            .localdb()
            .pkgs()
            .iter()
            .filter(|pkg| pkg.reason() == PackageReason::Explicit && pkg.required_by().is_empty())
            .map(|pkg| Item(pkg))
    }
}


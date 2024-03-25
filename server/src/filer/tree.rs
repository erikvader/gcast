use crate::{filer::cache::Pointer, util::basename};

use super::cache::{Cache, CacheDirEntry};

pub struct Tree<'a> {
    cache: &'a Cache,
    path: Vec<&'a CacheDirEntry>,
    root: Option<usize>, // None iff path is empty
}

pub enum Type {
    Regular,
    Directory,
    Root,
}

pub struct File<'a> {
    pub name: &'a str,
    pub path_relative_root: &'a str,
    pub ty: Type,
    pub id: usize,
}

impl<'a> Tree<'a> {
    pub fn new(cache: &'a Cache) -> Self {
        Self {
            cache,
            path: Vec::new(),
            root: None,
        }
    }

    pub fn files(&self) -> impl Iterator<Item = File<'_>> {
        let pointers = self.top_pointers();

        pointers.into_iter().map(|point| {
            let entry = self
                .cache
                .deref(*point)
                .expect("all pointers in the cache are valid");
            let relative_root = entry.path_relative_root();
            let (id, ty) = match point {
                Pointer::File(i) => (*i, Type::Regular),
                Pointer::Dir(i) => (*i, Type::Directory),
            };

            if entry.is_root() {
                File {
                    ty: Type::Root,
                    name: self.cache.root_path(entry),
                    path_relative_root: relative_root,
                    id,
                }
            } else {
                let name = basename(relative_root).expect("is not root");
                File {
                    ty,
                    name,
                    path_relative_root: relative_root,
                    id,
                }
            }
        })
    }

    pub fn cd_up(&mut self) -> Result<(), ()> {
        if self.path.is_empty() {
            return Err(());
        }

        self.path.pop();
        if self.path.is_empty() {
            self.root = None;
        }
        Ok(())
    }

    pub fn cd(&mut self, dir_id: usize) -> Result<(), ()> {
        let dir_pointer = Pointer::Dir(dir_id);

        let Some(entry) = self.cache.deref_dir(dir_pointer) else {
            return Err(());
        };

        if self.path.is_empty() {
            self.root = Some(entry.root());
        }
        self.path.push(entry);

        Ok(())
    }

    pub fn root(&self) -> Option<usize> {
        self.root
    }

    pub fn breadcrumbs(&self) -> Vec<String> {
        let mut bread = Vec::new();

        if let Some(direntry) = self.path.first() {
            bread.push(self.cache.root_path(direntry.cache_entry()).to_string());
        }

        bread.extend(self.path.iter().skip(1).map(|p| {
            basename(p.path_relative_root())
                .map(str::to_string)
                .unwrap_or_else(|| format!("??"))
        }));

        bread
    }

    fn top_pointers(&self) -> &[Pointer] {
        match self.path[..] {
            [] => self.cache.roots(),
            [.., top] => top.children(),
        }
    }
}

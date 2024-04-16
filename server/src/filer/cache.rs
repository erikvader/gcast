mod scan;

use itertools::Itertools;
use std::time::SystemTime;

pub use scan::{read_cache, refresh_cache, write_cache};

/// A cache of all files and directories from a list of source directories called "roots".
/// The vectors in this struct are sorted in some "standard" order, which in this case
/// means: ascending order by their path. The vectors can not be modified, since there are
/// `Pointer`s and other `usize`s pointing to locations in the vectors.
///
/// All paths are required to be valid rust strings, i.e., be valid UTF-8. This makes it
/// easier to use on the client, and the libmpv crate (v2.0.1) needs them to be `String`s
/// anyway. But this is a limitation that should be fixed in the future, i.e., use
/// `PathBuf` instead.
// TODO: make this partially updateable, i.e., update the files of a subfolder
#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Cache {
    /// All files found, sorted in ascending order by their path relative to their
    /// respective root.
    files: Vec<CacheEntry>,
    /// All dirs found, sorted in ascending order by their path relative to the respective
    /// root.
    dirs: Vec<CacheDirEntry>,
    /// The top most psuedo-`CacheDirEntry` containing pointers to all roots in `Dirs`.
    root_dir: Vec<Pointer>,
    /// The date when this cache was created.
    updated: Option<SystemTime>,
    /// The paths of all roots
    roots: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct CacheEntry {
    relative_path: String,
    root: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct CacheDirEntry {
    entry: CacheEntry,
    children: Vec<Pointer>,
}

// NOTE: mainly here for less copying in the function `link`.
#[derive(Debug, Hash, PartialEq, Eq)]
struct CacheEntryBorrowed<'a> {
    relative_path: &'a str,
    root: usize,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum Pointer {
    File(usize),
    Dir(usize),
}

impl CacheEntry {
    fn new(relative_path: String, root: usize) -> Self {
        assert!(relative_path.starts_with("/"));
        Self {
            relative_path,
            root,
        }
    }

    fn new_root(root: usize) -> Self {
        Self::new("/".to_string(), root)
    }

    pub(super) fn is_root(&self) -> bool {
        self.relative_path == "/"
    }

    pub(super) fn root(&self) -> usize {
        self.root
    }

    pub(super) fn path_relative_root(&self) -> &str {
        &self.relative_path
    }

    /// The character index in `path_relative_root` where the basename starts, i.e., the
    /// last path separator.
    pub(super) fn basename_char(&self) -> usize {
        match self
            .path_relative_root()
            .chars()
            .enumerate()
            .filter(|&(_, c)| c == std::path::MAIN_SEPARATOR)
            .last()
        {
            Some((i, _)) => i,
            None => 0,
        }
    }

    fn parent(&self) -> Option<CacheEntryBorrowed<'_>> {
        if self.is_root() {
            return None;
        }

        Some(CacheEntryBorrowed {
            root: self.root,
            relative_path: crate::util::dirname(&self.relative_path).unwrap_or("/"),
        })
    }

    fn borrow(&self) -> CacheEntryBorrowed<'_> {
        CacheEntryBorrowed {
            root: self.root,
            relative_path: &self.relative_path,
        }
    }
}

impl CacheDirEntry {
    fn new(relative_path: String, root: usize) -> Self {
        Self {
            entry: CacheEntry::new(relative_path, root),
            children: vec![],
        }
    }

    fn new_root(root: usize) -> Self {
        Self {
            entry: CacheEntry::new_root(root),
            children: vec![],
        }
    }

    fn set_children(&mut self, children: Vec<Pointer>) {
        self.children = children;
    }

    delegate::delegate! {
        to self.entry {
            pub(super) fn root(&self) -> usize;
            #[allow(dead_code)]
            pub(super) fn is_root(&self) -> bool;
            pub(super) fn path_relative_root(&self) -> &str;
            #[call(borrow)]
            fn borrow_cache_entry(&self) -> CacheEntryBorrowed<'_>;
            fn parent(&self) -> Option<CacheEntryBorrowed<'_>>;
        }
    }

    pub(super) fn children(&self) -> &[Pointer] {
        &self.children
    }

    pub(super) fn cache_entry(&self) -> &CacheEntry {
        &self.entry
    }
}

impl AsRef<str> for CacheEntry {
    fn as_ref(&self) -> &str {
        self.path_relative_root()
    }
}

impl Cache {
    fn new(
        files: Vec<CacheEntry>,
        dirs: Vec<CacheDirEntry>,
        roots: Vec<String>,
        root_dir: Vec<Pointer>,
    ) -> Self {
        let cache = Self {
            files,
            dirs,
            updated: Some(SystemTime::now()),
            roots,
            root_dir,
        };

        assert!(
            cache
                .dirs
                .iter()
                .all(|dir| dir.children.windows(2).all(|pair| {
                    use Pointer::*;
                    match (pair[0], pair[1]) {
                        (File(_), Dir(_)) => false,
                        (Dir(_), File(_)) => true,
                        (l, r) => match (cache.deref(l), cache.deref(r)) {
                            (Some(l), Some(r)) => {
                                l.path_relative_root() <= r.path_relative_root()
                            }
                            _ => false,
                        },
                    }
                })),
            concat!(
                "wrong order in dirs children, dirs come first then files,",
                " each kind sorted with themselves by path_relative_root. Some pointer",
                " could also be invalid.",
            )
        );

        assert!(
            cache.files.iter().map(|ce| ce.borrow()).all_unique(),
            "duplicate files"
        );
        assert!(
            cache
                .dirs
                .iter()
                .map(|ce| ce.borrow_cache_entry())
                .all_unique(),
            "duplicate dirs"
        );

        assert!(
            cache
                .files
                .windows(2)
                .all(|pair| pair[0].path_relative_root() <= pair[1].path_relative_root()),
            "files not sorted correctly"
        );

        assert!(
            cache
                .dirs
                .windows(2)
                .all(|pair| pair[0].path_relative_root() <= pair[1].path_relative_root()),
            "dirs not sorted correctly"
        );

        assert!(
            cache.root_dir.windows(2).all(|pair| {
                let l = cache.deref(pair[0]);
                let r = cache.deref(pair[1]);
                match (l, r) {
                    (Some(l), Some(r)) => {
                        l.path_relative_root() <= r.path_relative_root()
                    }
                    _ => false,
                }
            }),
            "psuedo-root dir not sorted correctly, or some pointers are invalid"
        );

        cache
    }

    pub fn updated(&self) -> Option<SystemTime> {
        self.updated
    }

    /// Retrieves all files sorted by their paths relative to their respective roots.
    pub(super) fn files(&self) -> impl Iterator<Item = &CacheEntry> {
        self.files.iter()
    }

    pub(super) fn roots(&self) -> &[Pointer] {
        &self.root_dir
    }

    pub fn roots_path(&self) -> &[String] {
        &self.roots
    }

    pub fn is_outdated(&self, roots: &[String]) -> bool {
        self.roots != roots
    }

    pub(super) fn deref(&self, pointer: Pointer) -> Option<&CacheEntry> {
        match pointer {
            Pointer::Dir(i) => self.dirs.get(i).map(CacheDirEntry::cache_entry),
            Pointer::File(i) => self.files.get(i),
        }
    }

    pub(super) fn deref_dir(&self, pointer: Pointer) -> Option<&CacheDirEntry> {
        match pointer {
            Pointer::Dir(i) => self.dirs.get(i),
            Pointer::File(_) => None,
        }
    }

    #[allow(dead_code)]
    pub(super) fn deref_file(&self, pointer: Pointer) -> Option<&CacheEntry> {
        match pointer {
            Pointer::Dir(_) => None,
            Pointer::File(i) => self.files.get(i),
        }
    }

    pub(super) fn root_path(&self, entry: &CacheEntry) -> &str {
        let i = entry.root();
        self.roots
            .get(i)
            .map(String::as_str)
            .expect("there will always be a root (lich king)")
    }
}

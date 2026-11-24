use rspack_paths::{InternedPath, InternedPathDashMap, InternedPathMap};

pub(super) enum MemberKind {
  File,
  Directory,
}

// An initial scan can follow a directory symlink that a live event records as a file.
#[derive(Clone, Copy, Default)]
pub(super) struct ContextMember {
  pub(super) file: bool,
  pub(super) directory: bool,
}

#[derive(Default)]
pub(super) struct DirectoryWatcher {
  pub(super) members: InternedPathMap<ContextMember>,
}

#[derive(Default)]
pub(super) struct WatcherManager {
  pub(super) directories: InternedPathDashMap<DirectoryWatcher>,
}

impl WatcherManager {
  pub(super) fn add_member(&self, path: &InternedPath, kind: MemberKind) -> bool {
    let parent = InternedPath::from(path.parent().expect("a context member has a parent"));
    let mut watcher = self.directories.entry(parent).or_default();
    let member = watcher.members.entry(path.clone()).or_default();
    let present = match kind {
      MemberKind::File => &mut member.file,
      MemberKind::Directory => &mut member.directory,
    };
    !std::mem::replace(present, true)
  }

  pub(super) fn member(&self, path: &InternedPath) -> Option<ContextMember> {
    let parent = InternedPath::from(path.parent()?);
    self.directories.get(&parent)?.members.get(path).copied()
  }

  pub(super) fn is_directory_member(&self, path: &InternedPath) -> bool {
    self.member(path).is_some_and(|member| member.directory)
  }

  pub(super) fn remove_member(&self, path: &InternedPath) -> ContextMember {
    let mut removed = ContextMember::default();
    if let Some(parent) = path.parent() {
      self
        .directories
        .remove_if_mut(&InternedPath::from(parent), |_, watcher| {
          removed = watcher.members.remove(path).unwrap_or_default();
          watcher.members.is_empty()
        });
    }
    removed
  }
}

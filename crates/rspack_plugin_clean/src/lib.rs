use std::sync::Arc;

use rspack_core::{
  ApplyContext, CleanOptions, Compilation, CompilationProcessAssets, CompilerOptions, Plugin,
  PluginContext,
};
use rspack_error::{error, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashSet;
use tracing::{debug, info, warn};

/// The CleanPlugin implementation
#[plugin]
#[derive(Debug, Clone)]
pub struct CleanPlugin {
  options: CleanOptions,
  /// Set of files that have been emitted by rspack
  emitted_assets: Arc<std::sync::Mutex<FxHashSet<String>>>,
  /// Whether the initial clean has been performed
  initial_clean_done: Arc<std::sync::Mutex<bool>>,
}

impl CleanPlugin {
  /// Create a new CleanPlugin with the given options
  pub fn new(options: CleanOptions) -> Self {
    Self::new_inner(
      options,
      Arc::new(std::sync::Mutex::new(FxHashSet::default())),
      Arc::new(std::sync::Mutex::new(false)),
    )
  }

  /// Perform the actual cleaning operation
  async fn clean_output_path(&self, output_path: &Utf8PathBuf) -> Result<()> {
    if !output_path.exists() {
      debug!("Output path does not exist: {}", output_path);
      return Ok(());
    }

    let files_to_remove = self.collect_files_to_clean(output_path).await?;

    // Log what will be done
    if self.options.dry {
      info!("[CleanPlugin] Dry run - files that would be removed:");
      for file in &files_to_remove {
        info!("  - {}", file);
      }
    } else {
      // Actually remove the files
      for file_path in &files_to_remove {
        if let Err(e) = std::fs::remove_file(file_path) {
          warn!("Failed to remove file {}: {}", file_path, e);
        } else {
          debug!("Removed file: {}", file_path);
        }
      }

      // Remove empty directories
      self.remove_empty_directories(output_path)?;
    }

    Ok(())
  }

  /// Collect files that should be cleaned
  async fn collect_files_to_clean(&self, output_path: &Utf8PathBuf) -> Result<Vec<Utf8PathBuf>> {
    let mut files_to_clean = Vec::new();
    self
      .collect_files_recursive(output_path, output_path, &mut files_to_clean)
      .await?;
    Ok(files_to_clean)
  }

  /// Collect files that should be cleaned (non-recursive implementation)
  async fn collect_files_recursive(
    &self,
    output_path: &Utf8PathBuf,
    current_path: &Utf8PathBuf,
    files_to_clean: &mut Vec<Utf8PathBuf>,
  ) -> Result<()> {
    use std::collections::VecDeque;

    let mut queue = VecDeque::new();
    queue.push_back(current_path.clone());

    while let Some(path) = queue.pop_front() {
      let entries =
        std::fs::read_dir(&path).map_err(|e| error!("Failed to read directory: {}", e))?;

      for entry in entries {
        let entry = entry.map_err(|e| error!("Failed to read directory entry: {}", e))?;
        let entry_path =
          Utf8PathBuf::from_path_buf(entry.path()).map_err(|_| error!("Invalid UTF-8 path"))?;

        let relative_path = entry_path
          .strip_prefix(output_path)
          .map_err(|_| error!("Failed to get relative path"))?;

        let relative_path_str = relative_path.as_str();

        if entry
          .file_type()
          .map_err(|e| error!("Failed to get file type: {}", e))?
          .is_dir()
        {
          // Add directory to queue for processing
          queue.push_back(entry_path);
        } else {
          // Check if this file should be kept
          let should_keep = self.should_keep_file(relative_path_str).await?;

          if !should_keep {
            files_to_clean.push(entry_path);
          }
        }
      }
    }

    Ok(())
  }

  /// Check if a file should be kept based on the keep function
  async fn should_keep_file(&self, relative_path: &str) -> Result<bool> {
    // Check the keep function from options
    if let Some(keep_fn) = &self.options.keep {
      return keep_fn(relative_path).await;
    }

    // Check if this file was emitted by webpack
    let emitted_assets = self
      .emitted_assets
      .lock()
      .expect("Failed to lock emitted_assets");
    if emitted_assets.contains(relative_path) {
      return Ok(true);
    }

    Ok(false)
  }

  /// Remove empty directories recursively
  #[allow(clippy::only_used_in_recursion)]
  fn remove_empty_directories(&self, path: &Utf8PathBuf) -> Result<()> {
    let entries = std::fs::read_dir(path).map_err(|e| error!("Failed to read directory: {}", e))?;

    for entry in entries {
      let entry = entry.map_err(|e| error!("Failed to read directory entry: {}", e))?;
      let entry_path =
        Utf8PathBuf::from_path_buf(entry.path()).map_err(|_| error!("Invalid UTF-8 path"))?;

      if entry
        .file_type()
        .map_err(|e| error!("Failed to get file type: {}", e))?
        .is_dir()
      {
        self.remove_empty_directories(&entry_path)?;

        // Check if directory is now empty after recursive cleanup
        if std::fs::read_dir(&entry_path)
          .map_err(|e| error!("Failed to read directory: {}", e))?
          .next()
          .is_none()
        {
          std::fs::remove_dir(&entry_path)
            .map_err(|e| error!("Failed to remove empty directory {}: {}", entry_path, e))?;
        }
      }
    }

    Ok(())
  }
}

#[plugin_hook(CompilationProcessAssets for CleanPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let should_clean = {
    let mut initial_clean_done = self
      .initial_clean_done
      .lock()
      .expect("Failed to lock initial_clean_done");
    if !*initial_clean_done {
      *initial_clean_done = true;
      true
    } else {
      false
    }
  };

  if should_clean {
    // Get output path from compilation options
    let output_path = &compilation.options.output.path;
    if let Err(e) = self.clean_output_path(output_path).await {
      warn!("CleanPlugin error: {}", e);
    }
  }
  Ok(())
}

impl Plugin for CleanPlugin {
  fn name(&self) -> &'static str {
    "CleanPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));

    Ok(())
  }
}

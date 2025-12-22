use clap::{Parser, Subcommand};
use rspack_error::Diagnostic;
use rspack_kit::compare_cache_dir;
use rspack_paths::Utf8PathBuf;

/// Toolkit for debugging and testing rspack internals
#[derive(Parser, Debug)]
#[command(name = "rspack_kit")]
#[command(about = "Toolkit for debugging and testing rspack internals", long_about = None)]
#[command(version)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// Compare two rspack cache directories to verify they contain the same data
  Compare {
    /// Path to the first cache directory
    #[arg(value_name = "CACHE1")]
    cache1: String,

    /// Path to the second cache directory
    #[arg(value_name = "CACHE2")]
    cache2: String,
  },
}

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  match cli.command {
    Commands::Compare { cache1, cache2 } => {
      println!("Comparing cache directories:");
      println!("  Path 1: {}", cache1);
      println!("  Path 2: {}", cache2);
      println!();

      let path1 = Utf8PathBuf::from(&cache1);
      let path2 = Utf8PathBuf::from(&cache2);

      if let Err(err) = compare_cache_dir(path1, path2).await {
        eprintln!(
          "{}",
          Diagnostic::from(err)
            .render_report(true)
            .expect("render error failed")
        );
        std::process::exit(1);
      }

      println!("✓ Cache directories are identical");
    }
  }
}

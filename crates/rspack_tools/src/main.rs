use clap::{Parser, Subcommand};
use rspack_error::Diagnostic;
use rspack_paths::Utf8PathBuf;
use rspack_tools::{bench_diff, compare_cache_dir};

/// Toolkit for debugging and testing rspack internals
#[derive(Parser, Debug)]
#[command(name = "rspack_tools")]
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
  /// Compare two hotpath JSON benchmark reports
  BenchDiff {
    /// Path to the baseline hotpath JSON report
    #[arg(value_name = "BEFORE_JSON")]
    before_json: String,

    /// Path to the candidate hotpath JSON report
    #[arg(value_name = "AFTER_JSON")]
    after_json: String,
  },
}

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  match cli.command {
    Commands::Compare { cache1, cache2 } => {
      println!("Comparing cache directories:");
      println!("  Path 1: {cache1}");
      println!("  Path 2: {cache2}");
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
    Commands::BenchDiff {
      before_json,
      after_json,
    } => {
      let before_path = Utf8PathBuf::from(&before_json);
      let after_path = Utf8PathBuf::from(&after_json);

      match bench_diff(before_path, after_path) {
        Ok(output) => {
          print!("{output}");
        }
        Err(err) => {
          eprintln!(
            "{}",
            Diagnostic::from(err)
              .render_report(true)
              .expect("render error failed")
          );
          std::process::exit(1);
        }
      }
    }
  }
}

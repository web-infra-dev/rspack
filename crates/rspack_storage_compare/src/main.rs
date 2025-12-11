use clap::Parser;
use rspack_error::Diagnostic;
use rspack_paths::Utf8PathBuf;
use rspack_storage_compare::compare_storage_dirs;

/// Compare two rspack cache directories
#[derive(Parser, Debug)]
#[command(name = "rspack_storage_compare")]
#[command(about = "Compare two rspack cache directories to verify they contain the same data", long_about = None)]
struct Args {
  /// Path to the first cache directory
  #[arg(long, value_name = "PATH")]
  cache1: String,

  /// Path to the second cache directory
  #[arg(long, value_name = "PATH")]
  cache2: String,
}

#[tokio::main]
async fn main() {
  let args = Args::parse();

  println!("Comparing cache directories:");
  println!("  Path 1: {}", args.cache1);
  println!("  Path 2: {}", args.cache2);
  println!();

  let path1 = Utf8PathBuf::from(&args.cache1);
  let path2 = Utf8PathBuf::from(&args.cache2);

  if let Err(err) = compare_storage_dirs(path1, path2).await {
    println!(
      "{}",
      Diagnostic::from(err)
        .render_report(true)
        .expect("render error failed")
    );
    std::process::exit(1);
  } else {
    std::process::exit(0);
  };
}

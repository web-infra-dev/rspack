fn main() {
  rspack_binding_build::setup();
  setup_node_zlib();

  #[cfg(feature = "sftrace-setup")]
  {
    sftrace_setup();
  }
}

#[cfg(feature = "sftrace-setup")]
fn sftrace_setup() {
  use std::path::PathBuf;

  fn search_sftracelib() -> Option<PathBuf> {
    use std::process::{Command, Stdio};

    let result = Command::new("sftrace")
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .arg("record")
      .arg("--print-solib")
      .output();

    match result {
      Ok(output) if output.status.success() => {
        let out = String::from_utf8(output.stdout).ok()?;
        let mut out = PathBuf::from(out);
        out.pop();
        Some(out)
      }
      _ => None,
    }
  }

  if let Some(lib) = search_sftracelib() {
    println!("cargo::rustc-link-arg=-Wl,-rpath,{}", lib.display());
  } else {
    println!("cargo::warning=not found sftrace");
  }
}

// The binding build command skips libz-sys's build script. Unix resolves these
// symbols from the host process; MSVC needs an import library for node.exe.
fn setup_node_zlib() {
  println!("cargo:rerun-if-env-changed=RSPACK_NODE_ZLIB");
  if std::env::var_os("RSPACK_NODE_ZLIB").is_none()
    || std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows")
  {
    return;
  }
  let target = std::env::var("TARGET").expect("TARGET is set by Cargo");
  let machine = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
    Ok("x86_64") => "X64",
    Ok("x86") => "X86",
    Ok("aarch64") => "ARM64",
    _ => panic!("Unsupported Node.js zlib target: {target}"),
  };
  let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
  let definition = out.join("node_zlib.def");
  std::fs::write(&definition, include_str!("node_zlib.def")).expect("write Node.js zlib exports");
  println!("cargo:rerun-if-changed=node_zlib.def");
  let library = out.join("node_zlib.lib");
  let mut librarian = cc::windows_registry::find_tool(&target, "lib.exe")
    .expect("MSVC lib.exe is required to link Node.js zlib")
    .to_command();
  let status = librarian
    .arg("/NOLOGO")
    .arg(format!("/MACHINE:{machine}"))
    .arg(format!("/DEF:{}", definition.display()))
    .arg(format!("/OUT:{}", library.display()))
    .status()
    .expect("run MSVC lib.exe");
  assert!(
    status.success(),
    "Failed to create Node.js zlib import library"
  );
  println!("cargo:rustc-link-arg-cdylib={}", library.display());
}

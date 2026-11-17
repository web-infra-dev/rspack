use rspack_core::{
  PortableRoot,
  cache::{CacheCodec, RootBinding},
};

const ROOT_DIR: &str = "lönger-读者-😀";

/// `ROOT_DIR` as each tool writes it in a string literal.
const ESCAPED_ROOT_DIRS: [&str; 11] = [
  // SWC, `jsc.target: 'es5'`
  r"l\xf6nger-读者-\uD83D\uDE00",
  // SWC, `jsc.output.charset: 'ascii'`, ES5 and ES2015
  r"l\xf6nger-\u8BFB\u8005-\uD83D\uDE00",
  r"l\xf6nger-\u8BFB\u8005-\u{1F600}",
  // Terser `ascii_only`, ES5 and ES2015
  r"l\xf6nger-\u8bfb\u8005-\ud83d\ude00",
  r"l\xf6nger-\u8bfb\u8005-\u{1f600}",
  // esbuild and Babel, ES5 and ES2015
  r"l\xF6nger-\u8BFB\u8005-\uD83D\uDE00",
  r"l\xF6nger-\u8BFB\u8005-\u{1F600}",
  // Python `json.dumps`
  r"l\u00f6nger-\u8bfb\u8005-\ud83d\ude00",
  // The TypeScript printer
  r"l\u00F6nger-\u8BFB\u8005-\uD83D\uDE00",
  // `\uHHHH` for every BMP character and `\u{H...}` above it
  r"l\u00f6nger-\u8bfb\u8005-\u{1f600}",
  r"l\u00F6nger-\u8BFB\u8005-\u{1F600}",
];

fn is_bound(root: &str, value: &str) -> bool {
  let codec = CacheCodec::new(Some(PortableRoot {
    root: root.into(),
    canonical_root: root.into(),
  }));
  let (_, binding) = codec
    .encode_with_binding(&value.to_string())
    .expect("a string encodes");
  matches!(binding, RootBinding::Bound)
}

fn nested_in_json(escaped: &str) -> String {
  escaped.split('\\').collect::<Vec<_>>().join(r"\\")
}

#[test]
fn ascii_escaped_root_is_bound() {
  let root = format!("/work/{ROOT_DIR}/proj");
  for dir in ESCAPED_ROOT_DIRS {
    let nested = nested_in_json(dir);
    for value in [
      format!("/work/{dir}/proj/src/a.jsx"),
      format!(r"\/work\/{dir}\/proj\/src\/a\.jsx"),
      format!("/work/{nested}/proj/src/a.jsx"),
      format!(r"\\/work\\/{nested}\\/proj\\/src\\/a.jsx"),
    ] {
      assert!(is_bound(&root, &value), "{value}");
    }
  }
}

#[test]
fn json_nested_regex_source_of_an_ascii_root_is_bound() {
  assert!(is_bound("/work/proj", r"\\/work\\/proj\\/src\\/a.js"));
}

#[test]
fn regex_escaped_root_is_bound() {
  for (root, value) in [
    ("/work/writer.plain", r"/^\/work\/writer\.plain\/src\//"),
    ("/work/writer.json", r"^\\/work\\/writer\\.json\\/src\\/"),
    (
      "/work/writer[bracket]",
      r"/^\/work\/writer\[bracket\]\/src\//",
    ),
    ("/work/my-app", r"/^\/work\/my\x2dapp\/src\//"),
    ("/work/my-app", r"/^\/work\/my\-app\/src\//"),
  ] {
    assert!(is_bound(root, value), "{value}");
  }
  assert!(!is_bound(
    "/work/writer.plain",
    r"/^\/work\/writerXplain\/src\//"
  ));
}

#[test]
fn ascii_escaped_windows_root_is_bound() {
  let root = format!(r"C:\work\{ROOT_DIR}\proj");
  for dir in ESCAPED_ROOT_DIRS {
    let nested = nested_in_json(dir);
    for value in [
      format!(r"C:\\work\\{dir}\\proj\\src\\a.jsx"),
      format!(r"c:\\work\\{dir}\\proj\\src\\a.jsx"),
      format!("C:/work/{dir}/proj/src/a.jsx"),
      format!(r"C:\/work\/{dir}\/proj\/src\/a.jsx"),
      format!(r"C:\\\\work\\\\{nested}\\\\proj\\\\src\\\\a.jsx"),
    ] {
      assert!(is_bound(&root, &value), "{value}");
    }
  }
}

#[test]
fn root_with_one_character_class_is_bound() {
  for (root, escaped) in [
    ("/work/ö/proj", r"/work/\xf6/proj/a.js"),
    ("/work/ö/proj", r"/work/\xF6/proj/a.js"),
    ("/work/ö/proj", r"/work/\u00f6/proj/a.js"),
    ("/work/ö/proj", r"/work/\u00F6/proj/a.js"),
    ("/work/读/proj", r"/work/\u8bfb/proj/a.js"),
    ("/work/读/proj", r"/work/\u8BFB/proj/a.js"),
    ("/work/😀/proj", r"/work/\ud83d\ude00/proj/a.js"),
    ("/work/😀/proj", r"/work/\uD83D\uDE00/proj/a.js"),
    ("/work/😀/proj", r"/work/\u{1f600}/proj/a.js"),
    ("/work/😀/proj", r"/work/\u{1F600}/proj/a.js"),
  ] {
    assert!(is_bound(root, escaped), "{escaped}");
  }
}

#[test]
fn escaped_look_alike_is_unbound() {
  let root = format!("/work/{ROOT_DIR}/proj");
  for value in [
    r"/work/l\xf7nger-读者-\uD83D\uDE00/proj/src/a.jsx",
    r"/work/l\xf6nger-读者-\uD83D\uDE01/proj/src/a.jsx",
    r"/work/l\xf6nger-读者-\uD83D\uDE00/other/src/a.jsx",
    r"/elsewhere/l\xf6nger-读者-\uD83D\uDE00/proj/src/a.jsx",
    r"/work/l\u00f6nger-\u8bfb\u8006-\u{1f600}/proj/src/a.jsx",
    r"\xf6nger-读者-\uD83D\uDE00",
  ] {
    assert!(!is_bound(&root, value), "{value}");
  }
}

#[test]
fn escaped_value_is_unbound_under_an_ascii_root() {
  assert!(!is_bound(
    "/work/plain/proj",
    r"/work/l\xf6nger/proj/src/a.jsx"
  ));
}

#[test]
fn base64_of_the_root_is_bound_at_every_alignment() {
  for value in [
    "eyJmaWxlIjoiL3dvcmsvcHJvamVjdC9zcmMvYS5qcyJ9",
    "eHsiZmlsZSI6Ii93b3JrL3Byb2plY3Qvc3JjL2EuanMifQ==",
    "eHh7ImZpbGUiOiIvd29yay9wcm9qZWN0L3NyYy9hLmpzIn0=",
  ] {
    assert!(is_bound("/work/project", value), "{value}");
  }
}

#[test]
fn base64_of_another_path_is_unbound() {
  assert!(!is_bound(
    "/work/project",
    "eyJmaWxlIjoiL3dvcmsvb3RoZXIvc3JjL2EuanMifQ=="
  ));
}

fn assert_base64_bound_at_every_alignment(root: &str, values: [&str; 3]) {
  // Base64 of `{"file":"<root>/index.js"}`, prefixed with "", "x" and "xx".
  let missed: Vec<_> = values
    .into_iter()
    .enumerate()
    .filter(|(_, value)| !is_bound(root, value))
    .collect();
  assert!(missed.is_empty(), "{root}: missed alignments {missed:?}");
}

#[test]
fn base64_of_a_four_byte_root_is_bound_at_every_alignment() {
  assert_base64_bound_at_every_alignment(
    "/app",
    [
      "eyJmaWxlIjoiL2FwcC9pbmRleC5qcyJ9",
      "eHsiZmlsZSI6Ii9hcHAvaW5kZXguanMifQ==",
      "eHh7ImZpbGUiOiIvYXBwL2luZGV4LmpzIn0=",
    ],
  );
}

#[test]
fn base64_of_a_five_byte_root_is_bound_at_every_alignment() {
  assert_base64_bound_at_every_alignment(
    "/work",
    [
      "eyJmaWxlIjoiL3dvcmsvaW5kZXguanMifQ==",
      "eHsiZmlsZSI6Ii93b3JrL2luZGV4LmpzIn0=",
      "eHh7ImZpbGUiOiIvd29yay9pbmRleC5qcyJ9",
    ],
  );
}

#[test]
fn base64_of_an_eight_byte_root_is_bound_at_every_alignment() {
  assert_base64_bound_at_every_alignment(
    "/tmp/258",
    [
      "eyJmaWxlIjoiL3RtcC8yNTgvaW5kZXguanMifQ==",
      "eHsiZmlsZSI6Ii90bXAvMjU4L2luZGV4LmpzIn0=",
      "eHh7ImZpbGUiOiIvdG1wLzI1OC9pbmRleC5qcyJ9",
    ],
  );
}

#[test]
fn base64_of_a_nine_byte_root_is_bound_at_every_alignment() {
  assert_base64_bound_at_every_alignment(
    "/tmp/2589",
    [
      "eyJmaWxlIjoiL3RtcC8yNTg5L2luZGV4LmpzIn0=",
      "eHsiZmlsZSI6Ii90bXAvMjU4OS9pbmRleC5qcyJ9",
      "eHh7ImZpbGUiOiIvdG1wLzI1ODkvaW5kZXguanMifQ==",
    ],
  );
}

#[test]
fn base64_of_a_ten_byte_root_is_bound_at_every_alignment() {
  assert_base64_bound_at_every_alignment(
    "/tmp/25890",
    [
      "eyJmaWxlIjoiL3RtcC8yNTg5MC9pbmRleC5qcyJ9",
      "eHsiZmlsZSI6Ii90bXAvMjU4OTAvaW5kZXguanMifQ==",
      "eHh7ImZpbGUiOiIvdG1wLzI1ODkwL2luZGV4LmpzIn0=",
    ],
  );
}

#[test]
fn base64_of_a_short_windows_root_is_bound_at_every_alignment() {
  // The backslash spelling is JSON-escaped inside the encoded payload.
  assert_base64_bound_at_every_alignment(
    r"C:\app",
    [
      "eyJmaWxlIjoiQzpcXGFwcFxcaW5kZXguanMifQ==",
      "eHsiZmlsZSI6IkM6XFxhcHBcXGluZGV4LmpzIn0=",
      "eHh7ImZpbGUiOiJDOlxcYXBwXFxpbmRleC5qcyJ9",
    ],
  );
}

#[test]
fn base64_of_a_short_windows_root_with_slashes_is_bound_at_every_alignment() {
  assert_base64_bound_at_every_alignment(
    r"C:\app",
    [
      "eyJmaWxlIjoiQzovYXBwL2luZGV4LmpzIn0=",
      "eHsiZmlsZSI6IkM6L2FwcC9pbmRleC5qcyJ9",
      "eHh7ImZpbGUiOiJDOi9hcHAvaW5kZXguanMifQ==",
    ],
  );
}

#[test]
fn base64_of_a_windows_drive_root_is_bound_at_every_alignment() {
  assert_base64_bound_at_every_alignment(
    r"C:\",
    [
      "eyJmaWxlIjoiQzpcXGluZGV4LmpzIn0=",
      "eHsiZmlsZSI6IkM6XFxpbmRleC5qcyJ9",
      "eHh7ImZpbGUiOiJDOlxcaW5kZXguanMifQ==",
    ],
  );
  assert_base64_bound_at_every_alignment(
    r"C:\",
    [
      "eyJmaWxlIjoiQzovaW5kZXguanMifQ==",
      "eHsiZmlsZSI6IkM6L2luZGV4LmpzIn0=",
      "eHh7ImZpbGUiOiJDOi9pbmRleC5qcyJ9",
    ],
  );
}

#[test]
fn one_byte_root_binds_every_value_without_panicking() {
  // One base64 alignment has no character determined solely by a one-byte root.
  for value in ["", "unrelated", "eyJmaWxlIjoiL290aGVyL2luZGV4LmpzIn0="] {
    assert!(is_bound("/", value), "{value}");
  }
}

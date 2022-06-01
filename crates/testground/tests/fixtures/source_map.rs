use crate::common::compile_fixture;
use sourcemap::SourceMap;

#[tokio::test]
async fn inline() {
  let bundler = compile_fixture("source-map/inline").await;
  assert_eq!(bundler.bundle.context.assets.lock().unwrap().len(), 1);
  let code = bundler
    .bundle
    .context
    .assets
    .lock()
    .unwrap()
    .get(0)
    .expect("failed to generate bundle")
    .source
    .to_owned();

  let last_line = code.lines().last().unwrap();
  assert!(last_line.starts_with("//# sourceMappingURL=data:application/json;charset=utf-8;base64,"));
}

#[tokio::test]
async fn none() {
  let bundler = compile_fixture("source-map/none").await;
  assert_eq!(bundler.bundle.context.assets.lock().unwrap().len(), 1);
  let code = bundler
    .bundle
    .context
    .assets
    .lock()
    .unwrap()
    .get(0)
    .expect("failed to generate bundle")
    .source
    .to_owned();

  let last_line = code.lines().last().unwrap();
  assert!(!last_line.starts_with("//# sourceMappingURL="));
}

#[tokio::test]
async fn external() {
  let bundler = compile_fixture("source-map/external").await;
  assert_eq!(bundler.bundle.context.assets.lock().unwrap().len(), 2);
  bundler
    .bundle
    .context
    .assets
    .lock()
    .unwrap()
    .iter()
    .for_each(|e| match e.filename.as_str() {
      "main.js" => {
        let last_line = e.source.lines().last().unwrap();
        assert!(!last_line.starts_with("//# sourceMappingURL=main.js.map"));
      }
      "main.js.map" => {
        SourceMap::from_slice(e.source.as_bytes()).expect("failed to parse source map");
      }
      _ => unreachable!(),
    });
}

#[tokio::test]
async fn linked() {
  let bundler = compile_fixture("source-map/linked").await;
  assert_eq!(bundler.bundle.context.assets.lock().unwrap().len(), 2);
  bundler
    .bundle
    .context
    .assets
    .lock()
    .unwrap()
    .iter()
    .for_each(|e| match e.filename.as_str() {
      "main.js" => {
        let last_line = e.source.lines().last().unwrap();
        assert!(last_line.starts_with("//# sourceMappingURL=main.js.map"));
      }
      "main.js.map" => {
        SourceMap::from_slice(e.source.as_bytes()).expect("failed to parse source map");
      }
      _ => unreachable!(),
    });
}

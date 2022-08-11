macro_rules! run_loader {
  ($loader:expr, $resource:tt, $expected:tt) => {
    use rspack_loader_runner::*;
    use std::path::PathBuf;

    let mut cur_dir = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
    cur_dir = cur_dir.join("./tests/fixtures");
    cur_dir = cur_dir.canonicalize().unwrap();
    println!("{:#?}", cur_dir);

    let resource = cur_dir
      .join($resource)
      .canonicalize()
      .unwrap()
      .as_os_str()
      .to_str()
      .unwrap()
      .to_owned();

    let resource = "file://".to_owned() + &resource;

    let url = url::Url::parse(&resource).unwrap();
    let mut runner = LoaderRunner::new(
      ResourceData {
        resource: resource.to_owned(),
        resource_path: url.path().to_owned(),
        resource_query: url.query().map(|q| q.to_owned()),
        resource_fragment: url.fragment().map(|f| f.to_owned()),
      },
      $loader,
    );

    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .unwrap()
      .block_on(async {
        let runner_result = runner.run().await.unwrap();
        similar_asserts::assert_eq!(
          runner_result.content.try_into_string().unwrap(),
          $expected.to_owned()
        );
      });
  };
}

mod fixtures {
  use anyhow::Result;
  use rspack_loader_runner::*;

  pub struct SimpleLoader {}

  #[async_trait::async_trait]
  impl Loader for SimpleLoader {
    fn name(&self) -> &'static str {
      "basic-loader"
    }

    async fn run<'a>(&self, loader_context: &LoaderContext<'a>) -> Result<Option<LoaderResult>> {
      let source = loader_context.source.to_owned().try_into_string()?;
      Ok(Some(LoaderResult {
        content: Content::String(format!(
          r#"{}
html {{
  margin: 0;
}}"#,
          source
        )),
      }))
    }
  }

  pub struct LoaderChain1 {}

  #[async_trait::async_trait]
  impl Loader for LoaderChain1 {
    fn name(&self) -> &'static str {
      "chain1-loader"
    }

    async fn run<'a>(&self, loader_context: &LoaderContext<'a>) -> Result<Option<LoaderResult>> {
      let source = loader_context.source.to_owned().try_into_string()?;
      Ok(Some(LoaderResult {
        content: Content::String(format!(
          r#"{}
console.log(2);"#,
          source.trim()
        )),
      }))
    }
  }

  pub struct LoaderChain2 {}

  #[async_trait::async_trait]
  impl Loader for LoaderChain2 {
    fn name(&self) -> &'static str {
      "chain1-loader"
    }

    async fn run<'a>(&self, loader_context: &LoaderContext<'a>) -> Result<Option<LoaderResult>> {
      let source = loader_context.source.to_owned().try_into_string()?;
      Ok(Some(LoaderResult {
        content: Content::String(format!(
          r#"{}
console.log(3);"#,
          source.trim()
        )),
      }))
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn should_run_single_loader() {
    run_loader!(
      vec![Box::new(super::fixtures::SimpleLoader {})],
      "simple.css",
      r#"body {
  background-color: #fff;
}
html {
  margin: 0;
}"#
    );
  }

  #[test]
  fn should_run_loader_chain_from_right_to_left() {
    run_loader!(
      vec![
        Box::new(super::fixtures::LoaderChain2 {}),
        Box::new(super::fixtures::LoaderChain1 {})
      ],
      "simple.js",
      r#"console.log(1);
console.log(2);
console.log(3);"#
    );
  }
}

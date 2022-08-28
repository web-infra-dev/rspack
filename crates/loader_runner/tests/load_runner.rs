macro_rules! fixtures {
  () => {{
    use std::path::PathBuf;

    let mut cur_dir = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
    cur_dir = cur_dir.join("./tests/fixtures");
    cur_dir = cur_dir.canonicalize().unwrap();

    cur_dir
  }};

  ($resource:tt) => {{
    let cur_dir = fixtures!();

    let resource = cur_dir
      .join($resource)
      .canonicalize()
      .unwrap()
      .as_os_str()
      .to_str()
      .unwrap()
      .to_owned();

    resource
  }};
}

macro_rules! run_loader {
  (@base, $loader:expr, $resource:tt, $expected:expr) => {{
    use rspack_loader_runner::*;

    let resource = "file://".to_owned() + &fixtures!($resource);

    let url = url::Url::parse(&resource).unwrap();
    LoaderRunner::new(
      ResourceData {
        resource: resource.to_owned(),
        resource_path: url.path().to_owned(),
        resource_query: url.query().map(|q| q.to_owned()),
        resource_fragment: url.fragment().map(|f| f.to_owned()),
      },
      vec![]
    )
  }};

  (@raw, $loader:expr, $resource:tt, $expected:expr) => {
    let runner = run_loader!(@base, $loader, $resource, $expected);

    tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
      let runner_result = runner.run($loader, &LoaderRunnerAdditionalContext {
        compiler: &(),
        compilation: &()
      }).await.unwrap();
      similar_asserts::assert_eq!(
        runner_result.content,
        $expected
      );
    });
  };

  ($loader:expr, $resource:tt, $expected:tt) => {
    let runner = run_loader!(@base, $loader, $resource, $expected);

    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .unwrap()
      .block_on(async {
        let runner_result = runner.run($loader, &LoaderRunnerAdditionalContext {
          compiler: &(),
          compilation: &()
        }).await.unwrap();
        similar_asserts::assert_eq!(
          runner_result.content.try_into_string().unwrap(),
          $expected.to_owned()
        );
      });
  };
}

mod fixtures {
  use rspack_error::Result;
  use rspack_loader_runner::*;

  #[derive(Debug)]
  pub struct DirectPassLoader {}

  #[async_trait::async_trait]
  impl Loader<(), ()> for DirectPassLoader {
    fn name(&self) -> &'static str {
      "direct-pass-loader"
    }

    async fn run(
      &self,
      loader_context: &LoaderContext<'_, '_, (), ()>,
    ) -> Result<Option<LoaderResult>> {
      let source = loader_context.source.to_owned();
      Ok(Some(LoaderResult {
        content: source,
        meta_data: None,
      }))
    }

    fn as_any(&self) -> &dyn std::any::Any {
      self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
      self
    }
  }

  #[derive(Debug)]
  pub struct SimpleCssLoader {}

  #[async_trait::async_trait]
  impl Loader<(), ()> for SimpleCssLoader {
    fn name(&self) -> &'static str {
      "basic-loader"
    }

    async fn run(
      &self,
      loader_context: &LoaderContext<'_, '_, (), ()>,
    ) -> Result<Option<LoaderResult>> {
      let source = loader_context.source.to_owned().try_into_string()?;
      Ok(Some(LoaderResult {
        meta_data: None,
        content: Content::String(format!(
          r#"{}
html {{
  margin: 0;
}}"#,
          source
        )),
      }))
    }

    fn as_any(&self) -> &dyn std::any::Any {
      self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
      self
    }
  }

  #[derive(Debug)]
  pub struct LoaderChain1 {}

  #[async_trait::async_trait]
  impl Loader<(), ()> for LoaderChain1 {
    fn name(&self) -> &'static str {
      "chain-loader"
    }

    async fn run(
      &self,
      loader_context: &LoaderContext<'_, '_, (), ()>,
    ) -> Result<Option<LoaderResult>> {
      let source = loader_context.source.to_owned().try_into_string()?;
      Ok(Some(LoaderResult {
        content: Content::String(format!(
          r#"{}
console.log(2);"#,
          source.trim()
        )),
      }))
    }

    fn as_any(&self) -> &dyn std::any::Any {
      self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
      self
    }
  }

  #[derive(Debug)]
  pub struct LoaderChain2 {}

  #[async_trait::async_trait]
  impl Loader<(), ()> for LoaderChain2 {
    fn name(&self) -> &'static str {
      "chain-loader"
    }

    async fn run(
      &self,
      loader_context: &LoaderContext<'_, '_, (), ()>,
    ) -> Result<Option<LoaderResult>> {
      let source = loader_context.source.to_owned().try_into_string()?;
      Ok(Some(LoaderResult {
        content: Content::String(format!(
          r#"{}
console.log(3);"#,
          source.trim()
        )),
      }))
    }

    fn as_any(&self) -> &dyn std::any::Any {
      self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
      self
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn should_run_single_loader() {
    use rspack_loader_runner::*;

    let loaders: Vec<&dyn Loader<(), ()>> = vec![&super::fixtures::SimpleCssLoader {}];

    run_loader!(
      loaders,
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
    use rspack_loader_runner::*;

    let loaders: Vec<&dyn Loader<(), ()>> = vec![
      &super::fixtures::LoaderChain2 {},
      &super::fixtures::LoaderChain1 {},
    ];

    run_loader!(
      loaders,
      "simple.js",
      r#"console.log(1);
console.log(2);
console.log(3);"#
    );
  }

  #[test]
  fn should_work_with_binary_formatted_files() {
    use rspack_loader_runner::*;

    let expected = Content::from(std::fs::read(&fixtures!("file.png")).unwrap());
    let loaders: Vec<&dyn Loader<(), ()>> = vec![&super::fixtures::DirectPassLoader {}];

    run_loader!(
      @raw,
      loaders,
      "file.png",
      expected
    );
  }
}

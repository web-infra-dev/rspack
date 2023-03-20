macro_rules! fixtures {
  () => {{
    use std::path::PathBuf;

    let mut cur_dir = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").expect("TODO:"));
    cur_dir = cur_dir.join("./tests/fixtures");
    cur_dir = cur_dir.canonicalize().expect("TODO:");

    cur_dir
  }};

  ($resource:tt) => {{
    let cur_dir = fixtures!();

    let resource = cur_dir
      .join($resource)
      .canonicalize()
      .expect("TODO:")
      .as_os_str()
      .to_str()
      .expect("TODO:")
      .to_owned();

    resource
  }};
}

macro_rules! run_loader {
  (@base $loader:expr, $resource:tt, $expected:expr) => {{
    use rspack_loader_runner::*;

    let resource = format!("file://{}", fixtures!($resource));

    let url = url::Url::parse(&resource).expect("TODO:");
    LoaderRunner::new(
      ResourceData {
        resource: resource.to_owned(),
        resource_path: url.to_file_path().unwrap(),
        resource_query: url.query().map(|q| q.to_owned()),
        resource_fragment: url.fragment().map(|f| f.to_owned()),
        resource_description: None
      },
      vec![]
    )
  }};

  (@raw $loader:expr, $resource:tt, $expected:expr) => {
    let runner = run_loader!(@base $loader, $resource, $expected);

    tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("TODO:")
    .block_on(async {
      let (runner_result, _) = runner.run($loader, &LoaderRunnerAdditionalContext {
        compiler: &(),
        compilation: &()
      }).await.expect("TODO:").split_into_parts();
      similar_asserts::assert_eq!(
        runner_result.content,
        $expected
      );
    });
  };

  ($loader:expr, $resource:tt, $expected:tt) => {
    let runner = run_loader!(@base $loader, $resource, $expected);

    tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .expect("TODO:")
      .block_on(async {
        let (runner_result, _) = runner.run($loader, &LoaderRunnerAdditionalContext {
          compiler: &(),
          compilation: &()
        }).await.expect("TODO:").split_into_parts();
        similar_asserts::assert_eq!(
          runner_result.content.try_into_string().expect("TODO:"),
          $expected.to_owned()
        );
      });
  };
}

#[cfg(test)]
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

    async fn run(&self, loader_context: &mut LoaderContext<'_, '_, (), ()>) -> Result<()> {
      let content = loader_context.content.to_owned();
      loader_context.cacheable = true;
      loader_context.content = content;

      Ok(())
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

    async fn run(&self, loader_context: &mut LoaderContext<'_, '_, (), ()>) -> Result<()> {
      let content = loader_context.content.to_owned().try_into_string()?;
      loader_context.content = Content::String(format!(
        r#"{content}
html {{
  margin: 0;
}}"#
      ));
      Ok(())
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

    async fn run(&self, loader_context: &mut LoaderContext<'_, '_, (), ()>) -> Result<()> {
      let content = loader_context.content.to_owned().try_into_string()?;
      loader_context.content = Content::String(format!(
        r#"{content}
console.log(2);"#
      ));
      Ok(())
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

    async fn run(&self, loader_context: &mut LoaderContext<'_, '_, (), ()>) -> Result<()> {
      let content = loader_context.content.to_owned().try_into_string()?;
      loader_context.content = Content::String(format!(
        r#"{content}
console.log(3);"#
      ));
      Ok(())
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

    let expected = Content::from(std::fs::read(fixtures!("file.png")).expect("TODO:"));
    let loaders: Vec<&dyn Loader<(), ()>> = vec![&super::fixtures::DirectPassLoader {}];

    run_loader!(
      @raw
      loaders,
      "file.png",
      expected
    );
  }
}

mod client_reference_dependency;

use std::{
  collections::{HashMap, HashSet},
  fmt::format,
  sync::atomic::AtomicBool,
};

use client_reference_dependency::ClientReferenceDependency;
use regex::Regex;
use rspack_core::{
  dummy_js_ast_path,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  AssetInfo, BoxModuleDependency, Compilation, CompilationArgs, CompilationAsset,
  EsmDynamicImportDependency, Module, ModuleDependency, ParseContext, Plugin,
  PluginCompilationHookOutput, PluginContext, PluginProcessAssetsOutput,
  PluginThisCompilationHookOutput, ProcessAssetsArgs, ThisCompilationArgs,
};
use serde::{Deserialize, Serialize};

// This is the module that will be used to anchor all client references to.
// I.e. it will have all the client files as async deps from this point on.
// We use the Flight client implementation because you can't get to these
// without the client runtime so it's the first time in the loading sequence
// you might want them.
static client_import_name: &str = "react-server-dom-webpack/client";
// static client_file_name: &str = "";

#[derive(Debug)]
pub struct ReactFlightPlugin {
  client_references: Vec<String>,
  chunk_name: String,
  client_manifest_filename: String,
  ssr_manifest_filename: String,

  // customized props
  resolved_client_references: Vec<ClientReferenceDependency>,
  client_file_name_found: AtomicBool,
  client_file_name: String,
}

impl ReactFlightPlugin {
  pub fn new(client_references: Vec<String>, client_file_name: String) -> Self {
    Self {
      // client_references: vec![ClientReferenceSearchPath {
      //   directory: ".".to_string(),
      //   recursive: true,
      //   include: Regex::new("\\.(js|ts|jsx|tsx)$").expect("todo"),
      // }],
      resolved_client_references: Default::default(),
      client_references,
      chunk_name: "client[index]".to_string(),
      client_manifest_filename: "react-client-manifest.json".to_string(),
      ssr_manifest_filename: "react-ssr-manifest.json".to_string(),
      client_file_name_found: Default::default(),
      client_file_name,
    }
  }

  fn resolve_all_client_files(&self) -> Vec<ClientReferenceDependency> {
    self
      .client_references
      .iter()
      .cloned()
      .map(|req| ClientReferenceDependency::new(req))
      .collect()
  }
}

impl ReactFlightPlugin {
  async fn inner_before_compile(&mut self, compilation: &mut Compilation) {
    self.resolved_client_references = self.resolve_all_client_files();
  }

  async fn inner_this_compilation(&mut self, _compilation: &mut Compilation) {
    // DependencyType::Custom would fallback to NormalModuleFactory

    // We currently don't need this code
    // https://github.com/facebook/react/blob/16d053d592673dd5565d85109f259371b23f87e8/packages/react-server-dom-webpack/src/ReactFlightWebpackPlugin.js#L144-L147

    // TODO: Rspack doesn't support parser hook now, So I switch to a just-added
    // `after_parse` hook
  }

  fn inner_after_parse(&self, ctx: &ParseContext, dependencies: &mut Vec<BoxModuleDependency>) {
    // let Some(module) = module.as_normal_module_mut() else {
    //   return
    // };
    let module_resource = ctx.resource_data.resource.as_str();
    if module_resource == self.client_file_name {
      self
        .client_file_name_found
        .swap(true, std::sync::atomic::Ordering::SeqCst);
    } else {
      return;
    }

    if !self.resolved_client_references.is_empty() {
      self
        .resolved_client_references
        .iter()
        .enumerate()
        .for_each(|(i, dep)| {
          let chunk_name = Regex::new(r#"\[index\]"#)
            .expect("todo")
            .replace_all(&self.chunk_name, &i.to_string());
          let chunk_name = Regex::new(r#"\[request\]"#)
            .expect("todo")
            // static toPath(str) {
            //   if (typeof str !== "string") return "";
            //   return str
            //     .replace(/[^a-zA-Z0-9_!§$()=\-^°]+/g, "-")
            //     .replace(/^-|-$/g, "");
            // }
            // TODO: port following above and apply it on dep.user_request()
            .replace_all(&chunk_name, dep.user_request());

          // ReactFlightWebpackPlugin use `AsyncDependenciesBlock` here
          // Rspack doesn't have this abstraction, but we could use EsmDynamicImportDependency, which
          // is the same to `EsmDynamicImportDependency` in Rspack

          let block = EsmDynamicImportDependency::new(
            dep.request().to_string().into(),
            None,
            dummy_js_ast_path(),
            Some(chunk_name.to_string()),
          );

          dependencies.push(Box::new(block));
        })
    }
  }

  async fn inner_process_assets_stage_report(&mut self, compilation: &mut Compilation) {
    if !*self.client_file_name_found.get_mut() {
      panic!("Client runtime at {} was not found. React Server Components module map file {} was not created.", client_import_name, self.client_manifest_filename)
    }

    let resolved_client_files = self
      .resolved_client_references
      .iter()
      .map(|r| r.request())
      .collect::<HashSet<_>>();

    #[derive(Serialize, Deserialize)]
    struct ClientManifest {
      pub chunks: Vec<String>,
      pub id: String,
      pub name: String,
    }

    #[derive(Serialize, Deserialize)]
    struct SsrExportItem {
      pub specifier: String,
      pub name: String,
    }
    type SsrManifest = HashMap<String, HashMap<String, SsrExportItem>>;
    let mut client_manifest: HashMap<String, ClientManifest> = Default::default();
    let mut ssr_manifest: SsrManifest = Default::default();

    compilation
      .chunk_group_by_ukey
      .values()
      .for_each(|chunk_group| {
        let chunk_ids = chunk_group
          .chunks
          .iter()
          .map(|c| c.as_ref(&compilation.chunk_by_ukey).id.clone().expect("id"))
          .collect::<Vec<_>>();

        chunk_group.chunks.iter().for_each(|c| {
          let chunk = c.as_ref(&compilation.chunk_by_ukey);
          let chunk_modules = compilation
            .chunk_graph
            .get_chunk_modules(&chunk.ukey, &compilation.module_graph);
          chunk_modules.iter().for_each(|module| {
            let module_id = compilation
              .chunk_graph
              .get_module_id(module.identifier())
              .as_ref()
              .expect("must have a id");
            let Some(normal_module) = (&***module).as_normal_module() else {
              return
            };

            if !resolved_client_files
              .contains(normal_module.resource_resolved_data().resource.as_str())
            {
              return;
            }

            let file_url = format!(
              "file://{}",
              normal_module.resource_resolved_data().resource.as_str()
            );
            let parsed_file_url = url::Url::parse(&file_url);
            if let Ok(href) = parsed_file_url.as_ref().map(|s| s.as_str()) {
              let mut ssr_exports: HashMap<String, SsrExportItem> = Default::default();

              client_manifest.insert(
                href.to_string(),
                ClientManifest {
                  chunks: chunk_ids.clone(),
                  id: module_id.to_string(),
                  name: "*".to_string(),
                },
              );

              ssr_exports.insert(
                "*".to_string(),
                SsrExportItem {
                  specifier: href.to_string(),
                  name: "*".to_string(),
                },
              );

              client_manifest.insert(
                format!("{href}#"),
                ClientManifest {
                  chunks: chunk_ids.clone(),
                  id: module_id.to_string(),
                  name: "*".to_string(),
                },
              );

              ssr_exports.insert(
                "".to_string(),
                SsrExportItem {
                  specifier: href.to_string(),
                  name: "".to_string(),
                },
              );

              // println!(
              //   "provided_exports {:#?}",
              //   compilation.exports_info_map.get(&module.identifier())
              // );
              if let Some(provided_exports) = compilation.exports_info_map.get(&module.identifier())
              {
                provided_exports.iter().for_each(|export_info| {
                  let name = &export_info.name;
                  client_manifest.insert(
                    format!("{href}#{name}"),
                    ClientManifest {
                      chunks: chunk_ids.clone(),
                      id: module_id.clone(),
                      name: name.to_string(),
                    },
                  );
                  ssr_exports.insert(
                    name.to_string(),
                    SsrExportItem {
                      specifier: href.to_string(),
                      name: name.to_string(),
                    },
                  );
                })
              }

              ssr_manifest.insert(module_id.to_string(), ssr_exports);
            } else {
              eprintln!("parse failed for {:?}", file_url)
            }
          })
          // Rspack doesn't support concatenation currently.
          // We don't need this code
          // https://github.com/facebook/react/blob/16d053d592673dd5565d85109f259371b23f87e8/packages/react-server-dom-webpack/src/ReactFlightWebpackPlugin.js#L315-L318
        });
      });

    let client_output = serde_json::to_string_pretty(&client_manifest).unwrap();
    compilation.emit_asset(
      self.client_manifest_filename.clone(),
      CompilationAsset::new(
        Some(RawSource::Source(client_output).boxed()),
        AssetInfo {
          minimized: false,
          content_hash: None,
          development: false,
          hot_module_replacement: false,
          related: Default::default(),
        },
      ),
    );
    let ssr_output = serde_json::to_string_pretty(&ssr_manifest).unwrap();
    compilation.emit_asset(
      self.ssr_manifest_filename.clone(),
      CompilationAsset::new(
        Some(RawSource::Source(ssr_output).boxed()),
        AssetInfo {
          minimized: false,
          content_hash: None,
          development: false,
          hot_module_replacement: false,
          related: Default::default(),
        },
      ),
    );
  }
}

#[async_trait::async_trait]
impl Plugin for ReactFlightPlugin {
  // TODO: Rspack doesn't have `before_compile` hook
  async fn compilation(&mut self, args: CompilationArgs<'_>) -> PluginCompilationHookOutput {
    self.inner_before_compile(args.compilation).await;
    Ok(())
  }

  async fn this_compilation(
    &mut self,
    args: ThisCompilationArgs<'_>,
  ) -> PluginThisCompilationHookOutput {
    self.inner_this_compilation(args.this_compilation).await;
    Ok(())
  }

  fn after_parse(
    &self,
    ctx: &ParseContext,
    dependencies: &mut Vec<BoxModuleDependency>,
  ) -> rspack_error::Result<()> {
    self.inner_after_parse(ctx, dependencies);
    Ok(())
  }

  async fn process_assets_stage_report(
    &mut self,
    _ctx: PluginContext,
    args: ProcessAssetsArgs<'_>,
  ) -> PluginProcessAssetsOutput {
    self
      .inner_process_assets_stage_report(args.compilation)
      .await;
    Ok(())
  }
}

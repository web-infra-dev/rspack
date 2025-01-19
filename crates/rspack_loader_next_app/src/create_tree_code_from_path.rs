use std::{collections::HashSet, path::PathBuf};

use rspack_core::{CompilationId, LoaderContext, RunnerContext};
use rspack_error::{error_bail as bail, Result};
use rspack_util::{
  fx_hash::{BuildFxHasher, FxIndexMap},
  json_stringify,
};

use crate::{
  create_metadata_exports_code::create_metadata_exports_code,
  create_static_metadata_from_route::{create_static_metadata_from_route, CollectingMetadata},
  util::{
    create_absolute_path, is_app_builtin_not_found_page, is_directory, is_group_segment,
    normalize_parallel_key, read_dir_with_compilation_cache, resolver,
  },
};

const DEFAULT_GLOBAL_ERROR_PATH: &str = "next/dist/client/components/error-boundary";
const DEFAULT_LAYOUT_PATH: &str = "next/dist/client/components/default-layout";
const DEFAULT_NOT_FOUND_PATH: &str = "next/dist/client/components/not-found-error";
const DEFAULT_FORBIDDEN_PATH: &str = "next/dist/client/components/forbidden-error";
const DEFAULT_UNAUTHORIZED_PATH: &str = "next/dist/client/components/unauthorized-error";
const DEFAULT_PARALLEL_ROUTE_PATH: &str = "next/dist/client/components/parallel-route-default";

const APP_DIR_ALIAS: &str = "private-next-app-dir";
const PAGE_SEGMENT: &str = "page$";
const PARALLEL_CHILDREN_SEGMENT: &str = "children$";
const UNDERSCORE_NOT_FOUND_ROUTE: &str = "/_not-found";
const UNDERSCORE_NOT_FOUND_ROUTE_ENTRY: &str = "/_not-found/page";
const PAGE_SEGMENT_KEY: &str = "__PAGE__";
const DEFAULT_SEGMENT_KEY: &str = "__DEFAULT__";

const HTTP_ACCESS_FALLBACKS: [&str; 3] = ["not-found", "forbidden", "unauthorized"];
const NORMAL_FILE_TYPES: [&str; 5] = ["layout", "template", "error", "loading", "global-error"];

pub struct TreeCodeResult {
  pub code: String,
  pub pages: String,
  pub root_layout: Option<String>,
  pub global_error: String,
}

pub async fn create_tree_code_from_path(
  page_path: &str,
  page: &str,
  loader_context: &mut LoaderContext<RunnerContext>,
  page_extensions: &[String],
  base_path: &str,
  app_dir: &str,
  app_paths: &[String],
  collected_declarations: &mut Vec<(String, String)>,
) -> Result<TreeCodeResult> {
  let is_not_found_route = page == UNDERSCORE_NOT_FOUND_ROUTE_ENTRY;
  let is_default_not_found = is_app_builtin_not_found_page(page_path);
  let app_dir_prefix = if is_default_not_found {
    APP_DIR_ALIAS
  } else {
    page_path.split_once('/').map(|i| i.0).unwrap_or(page_path)
  };
  let mut pages = vec![];

  let mut root_layout = None;
  let mut global_error = None;
  let mut tree_code = create_subtree_props_from_segment_path(
    vec![],
    collected_declarations,
    app_dir_prefix,
    base_path,
    is_default_not_found,
    is_not_found_route,
    page_extensions,
    app_paths,
    app_dir,
    loader_context,
    &mut pages,
    &mut root_layout,
    &mut global_error,
  )
  .await?;
  tree_code += ".children;";

  let mut pages = json_stringify(&pages);
  pages += ";";

  Ok(TreeCodeResult {
    code: tree_code,
    pages,
    root_layout,
    global_error: global_error.unwrap_or(DEFAULT_GLOBAL_ERROR_PATH.to_string()),
  })
}

#[derive(Debug)]
enum Segments<'a> {
  Children(&'a str),
  ParallelRoute(&'a str, Vec<&'a str>),
}

fn resolve_parallel_segments<'a>(
  pathname: &str,
  app_paths: &'a [String],
) -> Result<Vec<Segments<'a>>> {
  let mut matched: Vec<Segments> = Vec::new();
  let mut matched_children_index: Option<usize> = None;
  let mut existing_children_path: Option<&str> = None;

  for app_path in app_paths {
    if app_path.starts_with(&(pathname.to_string() + "/")) {
      let rest: Vec<&str> = app_path[pathname.len() + 1..].split('/').collect();

      // It is the actual page, mark it specially.
      if rest.len() == 1 && rest[0] == "page" {
        existing_children_path = Some(app_path);
        matched_children_index = Some(matched.len());
        matched.push(Segments::Children(PAGE_SEGMENT));
        continue;
      }

      let is_parallel_route = rest[0].starts_with('@');
      if is_parallel_route {
        if rest.len() == 2 && rest[1] == "page" {
          matched.push(Segments::ParallelRoute(rest[0], vec![PAGE_SEGMENT]));
          continue;
        }
        let mut segments = vec![PARALLEL_CHILDREN_SEGMENT];
        segments.extend(&rest[1..]);
        matched.push(Segments::ParallelRoute(rest[0], segments));
        continue;
      }

      if let Some(existing_path) = existing_children_path
        && let Some(i) = matched_children_index
        && let Segments::Children(c) = matched[i]
        && c != rest[0]
      {
        let is_incoming_parallel_page = app_path.contains('@');
        let has_current_parallel_page = existing_path.contains('@');

        if is_incoming_parallel_page {
          continue;
        } else if !has_current_parallel_page && !is_incoming_parallel_page {
          bail!("You cannot have two parallel pages that resolve to the same path. Please check {} and {}. Refer to the route group docs for more information: https://nextjs.org/docs/app/building-your-application/routing/route-groups", existing_path, app_path);
        }
      }

      existing_children_path = Some(app_path);

      if !matched.is_empty()
        && let Some(i) = matched_children_index
        && let Segments::Children(c) = matched[i]
        && c == rest[0]
      {
        continue;
      }

      matched_children_index = Some(matched.len());
      matched.push(Segments::Children(rest[0]));
    }
  }

  Ok(matched)
}

async fn resolve_adjacent_parallel_segments(
  segment_path: &str,
  app_dir_prefix: &str,
  app_dir: &str,
  compilation_id: CompilationId,
) -> Result<Vec<String>> {
  let absolute_segment_path =
    create_absolute_path(app_dir, &format!("{}{}", app_dir_prefix, segment_path));

  if absolute_segment_path.is_empty() {
    return Ok(vec![]);
  }

  let segment_is_directory = is_directory(&absolute_segment_path).await;

  if !segment_is_directory {
    return Ok(vec![]);
  }

  // We need to resolve all parallel routes in this level.
  let mut parallel_segments: Vec<String> = vec!["children".to_string()];
  read_dir_with_compilation_cache(&absolute_segment_path, compilation_id, |results| {
    for (name, metadata) in results.iter() {
      if metadata.is_dir() && name.starts_with('@') {
        parallel_segments.push(name.to_string());
      }
    }
  })
  .await?;

  Ok(parallel_segments)
}

#[async_recursion::async_recursion]
async fn create_subtree_props_from_segment_path(
  segments: Vec<&str>,
  nested_collected_declarations: &mut Vec<(String, String)>,
  app_dir_prefix: &str,
  base_path: &str,
  is_default_not_found: bool,
  is_not_found_route: bool,
  page_extensions: &[String],
  app_paths: &[String],
  app_dir: &str,
  loader_context: &mut LoaderContext<RunnerContext>,
  pages: &mut Vec<String>,
  root_layout: &mut Option<String>,
  global_error: &mut Option<String>,
) -> Result<String> {
  let segment_path = segments.join("/");

  let mut props: FxIndexMap<&str, String> = Default::default();
  let is_root_layer = segments.is_empty();
  let is_root_layout_or_root_page = segments.len() <= 1;

  let mut parallel_segments: Vec<Segments> = vec![];
  if is_root_layer {
    parallel_segments.push(Segments::Children(""));
  } else {
    parallel_segments.extend(resolve_parallel_segments(&segment_path, app_paths)?);
  }

  let mut metadata: Option<CollectingMetadata> = None;
  let router_dir_path = format!("{}{}", app_dir_prefix, segment_path);
  let resolved_route_dir = if is_default_not_found {
    "".to_string()
  } else {
    create_absolute_path(app_dir, &router_dir_path)
  };

  if !resolved_route_dir.is_empty() {
    metadata = create_static_metadata_from_route(
      &resolved_route_dir,
      &segment_path,
      is_root_layout_or_root_page,
      &page_extensions,
      base_path,
      &app_dir,
      loader_context,
    )
    .await?;
  }

  for segment in parallel_segments {
    if matches!(segment, Segments::Children(PAGE_SEGMENT)) {
      let matched_page_path = format!("{}{}/page", app_dir_prefix, segment_path);

      let (resolved_page_path, missing_dependencies) = resolver(
        &matched_page_path,
        app_dir,
        page_extensions,
        loader_context.context.compilation_id,
      )
      .await?;
      loader_context
        .missing_dependencies
        .extend(missing_dependencies);
      if let Some(resolved_page_path) = resolved_page_path {
        pages.push(resolved_page_path.clone());

        let var_name = format!("page{}", nested_collected_declarations.len());
        nested_collected_declarations.push((var_name.clone(), resolved_page_path.clone()));

        props.insert(
          "children",
          format!(
            "['{}', {{}}, {{\npage: [{}, {}], {}\n}}]",
            PAGE_SEGMENT_KEY,
            var_name,
            json_stringify(&resolved_page_path),
            create_metadata_exports_code(&metadata)
          ),
        );
        continue;
      }
    }

    let mut sub_segment_path = segments.clone();
    if let Segments::ParallelRoute(parallel_key, _) = segment {
      sub_segment_path.push(parallel_key);
    }

    let normalized_parallel_segment: &str = match &segment {
      Segments::Children(s) => s,
      Segments::ParallelRoute(_, s) => &s[0],
    };

    if normalized_parallel_segment != PAGE_SEGMENT
      && normalized_parallel_segment != PARALLEL_CHILDREN_SEGMENT
    {
      sub_segment_path.push(normalized_parallel_segment);
    }

    let mut parallel_segment_path = sub_segment_path.join("/");
    let parallel_segment_path = if parallel_segment_path.ends_with('/') {
      parallel_segment_path
    } else {
      parallel_segment_path.push('/');
      parallel_segment_path
    };

    let file_paths: Vec<_> = futures::future::join_all(
      NORMAL_FILE_TYPES
        .iter()
        .chain(HTTP_ACCESS_FALLBACKS.iter())
        .map(|file| {
          let parallel_segment_path = &parallel_segment_path;
          let compilation_id = loader_context.context.compilation_id;
          async move {
            let result = resolver(
              &format!("{}{}{}", app_dir_prefix, parallel_segment_path, file),
              app_dir,
              page_extensions,
              compilation_id,
            )
            .await?;
            Ok::<(&str, (Option<String>, HashSet<PathBuf, BuildFxHasher>)), rspack_error::Error>((
              file, result,
            ))
          }
        }),
    )
    .await
    .into_iter()
    .try_collect()?;

    let mut defined_file_paths: FxIndexMap<&str, String> =
      FxIndexMap::from_iter(file_paths.into_iter().filter_map(
        |(file, (path, missing_dependencies))| {
          loader_context
            .missing_dependencies
            .extend(missing_dependencies);
          path.map(|p| (file, p))
        },
      ));

    let is_first_layer_group_route = segments.len() == 1
      && sub_segment_path
        .iter()
        .filter(|seg| is_group_segment(seg))
        .count()
        == 1;

    if is_root_layer || is_first_layer_group_route {
      for &ty in HTTP_ACCESS_FALLBACKS.iter() {
        let (root_fallback_file, missing_dependencies) = resolver(
          &format!("{}/{}", app_dir_prefix, ty),
          app_dir,
          page_extensions,
          loader_context.context.compilation_id,
        )
        .await?;
        loader_context
          .missing_dependencies
          .extend(missing_dependencies);
        let has_root_fallback_file = root_fallback_file.is_some();

        let has_layer_fallback_file = defined_file_paths.contains_key(ty);

        if !(has_root_fallback_file && is_first_layer_group_route) && !has_layer_fallback_file {
          let default_fallback_path = match ty {
            "not-found" => DEFAULT_NOT_FOUND_PATH,
            "forbidden" => DEFAULT_FORBIDDEN_PATH,
            "unauthorized" => DEFAULT_UNAUTHORIZED_PATH,
            _ => unreachable!(),
          };
          defined_file_paths.insert(ty, default_fallback_path.to_string());
        }
      }
    }

    if root_layout.is_none() {
      let layout_path = defined_file_paths.get("layout");
      *root_layout = layout_path.cloned();
      if is_default_not_found && layout_path.is_none() && root_layout.is_none() {
        *root_layout = Some(DEFAULT_LAYOUT_PATH.to_string());
        defined_file_paths.insert("layout", DEFAULT_LAYOUT_PATH.to_string());
      }
    }

    if global_error.is_none() {
      let (resolved_global_error_path, missing_dependencies) = resolver(
        &format!("{}/{}", app_dir_prefix, "global-error"),
        app_dir,
        page_extensions,
        loader_context.context.compilation_id,
      )
      .await?;
      loader_context
        .missing_dependencies
        .extend(missing_dependencies);
      if let Some(resolved_global_error_path) = resolved_global_error_path {
        *global_error = Some(resolved_global_error_path);
      }
    }

    let (parallel_key, parallel_segment_key) = match &segment {
      Segments::Children(s) => ("children", *s),
      Segments::ParallelRoute(parallel_key, vec) => (*parallel_key, vec[0]),
    };
    let parallel_segment_key = match parallel_segment_key {
      PARALLEL_CHILDREN_SEGMENT => "children",
      PAGE_SEGMENT => PAGE_SEGMENT_KEY,
      _ => parallel_segment_key,
    };

    let normalized_parallel_key = normalize_parallel_key(parallel_key);
    let subtree_code = if is_not_found_route && normalized_parallel_key == "children" {
      let not_found_path = defined_file_paths
        .get("not-found")
        .map(|s| s.as_str())
        .unwrap_or(DEFAULT_NOT_FOUND_PATH);

      let var_name = format!("notFound{}", nested_collected_declarations.len());

      let code = format!(
        "{{\nchildren: [{}, {{\nchildren: ['{}', {}, {{\npage: [{}, {}]\n}}]\n}}, {{}}]\n}}",
        json_stringify(UNDERSCORE_NOT_FOUND_ROUTE),
        PAGE_SEGMENT_KEY,
        "{}",
        &var_name,
        json_stringify(not_found_path)
      );

      nested_collected_declarations.push((var_name, not_found_path.to_string()));

      code
    } else {
      create_subtree_props_from_segment_path(
        sub_segment_path,
        nested_collected_declarations,
        app_dir_prefix,
        base_path,
        is_default_not_found,
        is_not_found_route,
        page_extensions,
        app_paths,
        app_dir,
        loader_context,
        pages,
        root_layout,
        global_error,
      )
      .await?
    };

    let modules_code = format!(
      "{{\n{} {}\n}}",
      defined_file_paths
        .iter()
        .map(|(file, file_path)| {
          let var_name = format!("module{}", nested_collected_declarations.len());
          nested_collected_declarations.push((var_name.clone(), file_path.clone()));
          format!("'{}': [{}, {}],", file, var_name, json_stringify(file_path))
        })
        .collect::<Vec<_>>()
        .join("\n"),
      create_metadata_exports_code(&metadata)
    );

    props.insert(
      normalized_parallel_key,
      format!(
        "[\n'{}',\n{},\n{}\n]",
        parallel_segment_key, subtree_code, modules_code
      ),
    );
  }

  let adjacent_parallel_segments = resolve_adjacent_parallel_segments(
    &segment_path,
    app_dir_prefix,
    app_dir,
    loader_context.context.compilation_id,
  )
  .await?;

  for adjacent_parallel_segment in &adjacent_parallel_segments {
    if !props.contains_key(&normalize_parallel_key(&adjacent_parallel_segment)) {
      let actual_segment = if adjacent_parallel_segment == "children" {
        "".to_string()
      } else {
        format!("/{}", &adjacent_parallel_segment)
      };

      let (default_path, missing_dependencies) = resolver(
        &format!(
          "{}{}{}/default",
          app_dir_prefix, segment_path, actual_segment
        ),
        app_dir,
        page_extensions,
        loader_context.context.compilation_id,
      )
      .await?;
      loader_context
        .missing_dependencies
        .extend(missing_dependencies);
      let default_path = default_path.unwrap_or_else(|| DEFAULT_PARALLEL_ROUTE_PATH.to_string());
      let json_stringified_default_path = json_stringify(&default_path);

      let var_name = format!("default{}", nested_collected_declarations.len());
      nested_collected_declarations.push((var_name.clone(), default_path));

      props.insert(
        normalize_parallel_key(&adjacent_parallel_segment),
        format!(
          "[\n'{}', {}, {{\ndefaultPage: [{}, {}]\n}}\n]",
          DEFAULT_SEGMENT_KEY, "{}", var_name, json_stringified_default_path,
        ),
      );
    }
  }

  Ok(format!(
    "{{\n{}\n}}",
    props
      .into_iter()
      .map(|(k, v)| format!("{k}: {v}"))
      .collect::<Vec<_>>()
      .join(",\n")
  ))
}

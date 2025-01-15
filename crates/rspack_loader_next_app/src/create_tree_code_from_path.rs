use std::path::{Path, PathBuf, MAIN_SEPARATOR};

use regex::Regex;
use rspack_core::{LoaderContext, RunnerContext};
use rspack_error::{error, error_bail as bail, Result};

const DEFAULT_GLOBAL_ERROR_PATH: &str = "next/dist/client/components/error-boundary";
const DEFAULT_LAYOUT_PATH: &str = "next/dist/client/components/default-layout";

const APP_DIR_ALIAS: &str = "private-next-app-dir";
const PAGE_SEGMENT: &str = "page$";
const PARALLEL_CHILDREN_SEGMENT: &str = "children$";
const UNDERSCORE_NOT_FOUND_ROUTE_ENTRY: &str = "/_not-found/page";

pub struct TreeCodeResult {
  code: String,
  pages: String,
  root_layout: Option<String>,
  global_error: String,
}

pub async fn create_tree_code_from_path(
  page_path: &str,
  page: &str,
  loader_context: &mut LoaderContext<RunnerContext>,
  page_extensions: &[String],
  base_path: &str,
  collected_declarations: &mut Vec<(String, String)>,
) -> Result<TreeCodeResult> {
  let is_not_found_route = page == UNDERSCORE_NOT_FOUND_ROUTE_ENTRY;
  let is_default_not_found = is_app_builtin_not_found_page(page_path);
  let app_dir_prefix = if is_default_not_found {
    APP_DIR_ALIAS
  } else {
    page_path.split_once('/').map(|i| i.0).unwrap_or(page_path)
  };

  let mut root_layout = None;
  let mut global_error = None;
  let tree_code = create_subtree_props_from_segment_path(
    vec![],
    collected_declarations,
    app_dir_prefix,
    base_path,
    is_default_not_found,
    page_extensions,
    &mut root_layout,
    &mut global_error,
  )
  .await?;

  Ok(TreeCodeResult {
    code: tree_code,
    pages: String::new(),
    root_layout,
    global_error: global_error.unwrap_or(DEFAULT_GLOBAL_ERROR_PATH.to_string()),
  })
}

pub fn is_app_builtin_not_found_page(page: &str) -> bool {
  let re = lazy_regex::regex!(r"next[\\/]dist[\\/]client[\\/]components[\\/]not-found-error");
  re.is_match(page)
}

fn create_absolute_path(app_dir: &str, path_to_turn_absolute: &str) -> String {
  let p = path_to_turn_absolute.replace("/", &MAIN_SEPARATOR.to_string());
  if let Some(p) = p.strip_prefix("private-next-app-dir") {
    format!("{}{}", app_dir, p)
  } else {
    p
  }
}

async fn metadata_resolver(
  dirname: &str,
  loader_context: &mut LoaderContext<RunnerContext>,
  filename: &str,
  exts: Vec<&str>,
  app_dir: &str,
) -> Option<String> {
  let absolute_dir = create_absolute_path(app_dir, dirname);

  let mut result: Option<String> = None;

  for ext in exts {
    // Compared to `resolver` above the exts do not have the `.` included already, so it's added here.
    let filename_with_ext = format!("{}.{}", filename, ext);
    let absolute_path_with_extension = format!(
      "{}{}{}",
      absolute_dir,
      std::path::MAIN_SEPARATOR,
      filename_with_ext
    );
    if result.is_none() && file_exists_in_directory(dirname, &filename_with_ext).await {
      result = Some(absolute_path_with_extension.clone());
    }
    // Call `add_missing_dependency` for all files even if they didn't match,
    // because they might be added or removed during development.
    loader_context
      .missing_dependencies
      .insert(PathBuf::from(absolute_path_with_extension));
  }

  result
}

async fn resolver(
  pathname: &str,
  loader_context: &mut LoaderContext<RunnerContext>,
  app_dir: &str,
  extensions: Vec<&str>,
) -> Option<String> {
  let absolute_path = create_absolute_path(app_dir, pathname);

  let filename_index = absolute_path.rfind(std::path::MAIN_SEPARATOR).unwrap_or(0);
  let dirname = &absolute_path[..filename_index];
  let filename = &absolute_path[filename_index + 1..];

  let mut result: Option<String> = None;

  for ext in extensions {
    let absolute_path_with_extension = format!("{}.{}", absolute_path, ext);
    if result.is_none() && file_exists_in_directory(dirname, &format!("{}.{}", filename, ext)).await
    {
      result = Some(absolute_path_with_extension.clone());
    }
    // Call `add_missing_dependency` for all files even if they didn't match,
    // because they might be added or removed during development.
    loader_context
      .missing_dependencies
      .insert(PathBuf::from(absolute_path_with_extension));
  }

  result
}

async fn file_exists_in_directory(dirname: &str, filename: &str) -> bool {
  let path = Path::new(dirname).join(filename);
  tokio::fs::metadata(path)
    .await
    .map(|m| m.is_file())
    .unwrap_or(false)
}

enum Segments<'a> {
  Children(&'a str),
  ParallelRoute(&'a str, Vec<&'a str>),
}

fn resolve_parallel_segments<'a>(
  pathname: &str,
  app_paths: Vec<&'a str>,
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
  let mut files = tokio::fs::read_dir(&absolute_segment_path).await.unwrap();
  let mut parallel_segments: Vec<String> = vec!["children".to_string()];

  while let Some(entry) = files.next_entry().await.map_err(|e| error!(e))? {
    // Make sure name starts with "@" and is a directory.
    if entry.metadata().await.map_err(|e| error!(e))?.is_dir()
      && let Some(name) = entry.file_name().to_str()
      && name.starts_with('@')
    {
      parallel_segments.push(name.to_string());
    }
  }

  Ok(parallel_segments)
}

async fn is_directory(path: &str) -> bool {
  tokio::fs::metadata(path)
    .await
    .map(|m| m.is_dir())
    .unwrap_or(false)
}

async fn create_subtree_props_from_segment_path(
  segments: Vec<&str>,
  nested_collected_declarations: &mut Vec<(String, String)>,
  app_dir_prefix: &str,
  base_path: &str,
  is_default_not_found: bool,
  page_extensions: &[String],
  root_layout: &mut Option<String>,
  global_error: &mut Option<String>,
) -> Result<String> {
  let segment_path = segments.join("/");
  todo!()

  // let mut props: HashMap<String, String> = HashMap::new();
  // let is_root_layer = segments.is_empty();
  // let is_root_layout_or_root_page = segments.len() <= 1;

  // let mut parallel_segments: Vec<(String, String)> = vec![];
  // if is_root_layer {
  //   parallel_segments.push(("children".to_string(), "".to_string()));
  // } else {
  //   parallel_segments.extend(resolve_parallel_segments(&segment_path));
  // }

  // let mut metadata: Option<Metadata> = None;
  // let router_dir_path = format!("{}{}", app_dir_prefix, segment_path);
  // let resolved_route_dir = if is_default_not_found {
  //   "".to_string()
  // } else {
  //   resolve_dir(&router_dir_path).await.unwrap_or_default()
  // };

  // if !resolved_route_dir.is_empty() {
  //   metadata = Some(
  //     create_static_metadata_from_route(
  //       &resolved_route_dir,
  //       base_path,
  //       &segment_path,
  //       metadata_resolver,
  //       is_root_layout_or_root_page,
  //       &page_extensions,
  //     )
  //     .await?,
  //   );
  // }

  // for (parallel_key, parallel_segment) in parallel_segments {
  //   if parallel_segment == PAGE_SEGMENT {
  //     let matched_page_path = format!(
  //       "{}{}{}",
  //       app_dir_prefix,
  //       segment_path,
  //       if parallel_key == "children" {
  //         ""
  //       } else {
  //         &format!("/{}", parallel_key)
  //       }
  //     );

  //     if let Some(resolved_page_path) = resolver(&matched_page_path).await {
  //       pages.push(resolved_page_path.clone());

  //       let var_name = format!("page{}", nested_collected_declarations.len());
  //       nested_collected_declarations.push((var_name.clone(), resolved_page_path.clone()));

  //       props.insert(
  //         normalize_parallel_key(&parallel_key),
  //         format!(
  //           "['{}', {}, {{ page: [{}, {}], {} }}]",
  //           PAGE_SEGMENT_KEY,
  //           "{}",
  //           var_name,
  //           serde_json::to_string(&resolved_page_path)?,
  //           create_metadata_exports_code(metadata.as_ref())
  //         ),
  //       );
  //       continue;
  //     }
  //   }

  //   let mut sub_segment_path = segments.clone();
  //   if parallel_key != "children" {
  //     sub_segment_path.push(&parallel_key);
  //   }

  //   let normalized_parallel_segment =
  //     if parallel_segment == PAGE_SEGMENT || parallel_segment == PARALLEL_CHILDREN_SEGMENT {
  //       parallel_segment
  //     } else {
  //       sub_segment_path.push(&parallel_segment);
  //       parallel_segment
  //     };

  //   let parallel_segment_path = sub_segment_path.join("/");

  //   let file_paths: Vec<_> = futures::future::join_all(FILE_TYPES.iter().map(|file| async {
  //     let resolved_path = resolver(&format!(
  //       "{}{}{}",
  //       app_dir_prefix,
  //       if parallel_segment_path.ends_with('/') {
  //         &parallel_segment_path
  //       } else {
  //         &format!("{}/", parallel_segment_path)
  //       },
  //       file
  //     ))
  //     .await;
  //     (file, resolved_path)
  //   }))
  //   .await;

  //   let defined_file_paths: Vec<_> = file_paths
  //     .into_iter()
  //     .filter_map(|(file, path)| path.map(|p| (file, p)))
  //     .collect();

  //   let existed_convention_names: HashSet<_> =
  //     defined_file_paths.iter().map(|(type_, _)| *type_).collect();

  //   let is_first_layer_group_route =
  //     segments.len() == 1 && sub_segment_path.iter().any(|seg| is_group_segment(seg));

  //   if is_root_layer || is_first_layer_group_route {
  //     for type_ in default_http_access_fallback_paths.keys() {
  //       let has_root_fallback_file = resolver(&format!("{}/{}", app_dir_prefix, FILE_TYPES[type_]))
  //         .await
  //         .is_some();
  //       let has_layer_fallback_file = existed_convention_names.contains(type_);

  //       if !(has_root_fallback_file && is_first_layer_group_route) && !has_layer_fallback_file {
  //         let default_fallback_path = default_http_access_fallback_paths[type_];
  //         defined_file_paths.push((type_, default_fallback_path));
  //       }
  //     }
  //   }

  //   if root_layout.is_none() {
  //     if let Some(layout_path) = defined_file_paths
  //       .iter()
  //       .find(|(type_, _)| *type_ == "layout")
  //       .map(|(_, path)| path)
  //     {
  //       root_layout = Some(layout_path.clone());

  //       if is_default_not_found && root_layout.is_none() {
  //         root_layout = Some(default_layout_path.clone());
  //         defined_file_paths.push(("layout", default_layout_path.clone()));
  //       }
  //     }
  //   }

  //   if global_error.is_none() {
  //     if let Some(resolved_global_error_path) =
  //       resolver(&format!("{}/{}", app_dir_prefix, GLOBAL_ERROR_FILE_TYPE)).await
  //     {
  //       global_error = Some(resolved_global_error_path);
  //     }
  //   }

  //   let mut parallel_segment_key = if parallel_segment == PARALLEL_CHILDREN_SEGMENT {
  //     "children".to_string()
  //   } else if parallel_segment == PAGE_SEGMENT {
  //     PAGE_SEGMENT_KEY.to_string()
  //   } else {
  //     parallel_segment
  //   };

  //   let normalized_parallel_key = normalize_parallel_key(&parallel_key);
  //   let subtree_code = if is_not_found_route && normalized_parallel_key == "children" {
  //     let not_found_path = defined_file_paths
  //       .iter()
  //       .find(|(type_, _)| *type_ == "not-found")
  //       .map(|(_, path)| path)
  //       .unwrap_or(&default_http_access_fallback_paths["not-found"]);

  //     let var_name = format!("notFound{}", nested_collected_declarations.len());
  //     nested_collected_declarations.push((var_name.clone(), not_found_path.clone()));

  //     format!(
  //       "{{ children: [{}, {{ children: ['{}', {}, {{ page: [{}, {}] }}] }}] }}",
  //       serde_json::to_string(&UNDERSCORE_NOT_FOUND_ROUTE)?,
  //       PAGE_SEGMENT_KEY,
  //       "{}",
  //       var_name,
  //       serde_json::to_string(not_found_path)?
  //     )
  //   } else {
  //     let page_subtree_code = create_subtree_props_from_segment_path(
  //       sub_segment_path,
  //       nested_collected_declarations,
  //       app_dir_prefix,
  //       base_path,
  //       is_default_not_found,
  //       page_extensions.clone(),
  //     )
  //     .await?;

  //     page_subtree_code["treeCode"].clone()
  //   };

  //   let modules_code = format!(
  //     "{{ {} {} }}",
  //     defined_file_paths
  //       .iter()
  //       .map(|(file, file_path)| {
  //         let var_name = format!("module{}", nested_collected_declarations.len());
  //         nested_collected_declarations.push((var_name.clone(), file_path.clone()));
  //         format!(
  //           "'{}': [{}, {}],",
  //           file,
  //           var_name,
  //           serde_json::to_string(file_path)?
  //         )
  //       })
  //       .collect::<Vec<_>>()
  //       .join("\n"),
  //     create_metadata_exports_code(metadata.as_ref())
  //   );

  //   props.insert(
  //     normalized_parallel_key,
  //     format!(
  //       "[ '{}', {}, {} ]",
  //       parallel_segment_key, subtree_code, modules_code
  //     ),
  //   );
  // }

  // let adjacent_parallel_segments = resolve_adjacent_parallel_segments(&segment_path).await;

  // for adjacent_parallel_segment in adjacent_parallel_segments {
  //   if !props.contains_key(&normalize_parallel_key(&adjacent_parallel_segment)) {
  //     let actual_segment = if adjacent_parallel_segment == "children" {
  //       "".to_string()
  //     } else {
  //       format!("/{}", adjacent_parallel_segment)
  //     };

  //     let default_path = resolver(&format!(
  //       "{}{}{}",
  //       app_dir_prefix, segment_path, actual_segment
  //     ))
  //     .await
  //     .unwrap_or(PARALLEL_ROUTE_DEFAULT_PATH.to_string());

  //     let var_name = format!("default{}", nested_collected_declarations.len());
  //     nested_collected_declarations.push((var_name.clone(), default_path.clone()));

  //     props.insert(
  //       normalize_parallel_key(&adjacent_parallel_segment),
  //       format!(
  //         "[ '{}', {}, {{ defaultPage: [{}, {}] }} ]",
  //         DEFAULT_SEGMENT_KEY,
  //         "{}",
  //         var_name,
  //         serde_json::to_string(&default_path)?
  //       ),
  //     );
  //   }
  // }

  // Ok(props)
}

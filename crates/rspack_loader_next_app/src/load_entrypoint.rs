use rspack_error::{error, Result};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::{
  fx_hash::{FxIndexMap, FxIndexSet},
  json_stringify,
};
use sugar_path::SugarPath;

fn next_js_template_path(package_root: &Utf8Path) -> Utf8PathBuf {
  package_root
    .join("next")
    .join("dist")
    .join("esm")
    .join("build")
    .join("templates")
}

pub async fn load_next_js_template(
  path: &str,
  package_root: &Utf8Path,
  replacements: FxIndexMap<&'static str, String>,
  injections: FxIndexMap<&'static str, String>,
  imports: FxIndexMap<&'static str, Option<String>>,
) -> Result<String> {
  let path = next_js_template_path(package_root).join(path);

  let content = tokio::fs::read_to_string(&path)
    .await
    .map_err(|e| error!(e))?;

  let parent_path = path.parent().expect("should have parent path");

  fn replace_all<E>(
    re: &regex::Regex,
    haystack: &str,
    mut replacement: impl FnMut(&regex::Captures) -> Result<String, E>,
  ) -> Result<String, E> {
    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;
    for caps in re.captures_iter(haystack) {
      let m = caps.get(0).unwrap();
      new.push_str(&haystack[last_match..m.start()]);
      new.push_str(&replacement(&caps)?);
      last_match = m.end();
    }
    new.push_str(&haystack[last_match..]);
    Ok(new)
  }

  // Update the relative imports to be absolute. This will update any relative
  // imports to be relative to the root of the `next` package.
  let regex = lazy_regex::regex!("(?:from '(\\..*)'|import '(\\..*)')");

  let mut count = 0;
  let mut content = replace_all(regex, &content, |caps| {
    let from_request = caps.get(1).map_or("", |c| c.as_str());
    let import_request = caps.get(2).map_or("", |c| c.as_str());

    count += 1;
    let is_from_request = !from_request.is_empty();

    let imported = parent_path.join(if is_from_request {
      from_request
    } else {
      import_request
    });

    let relative = imported.as_std_path().relative(package_root.as_std_path());

    if !relative.starts_with("next/") {
      return Err(error!(
        "Invariant: Expected relative import to start with \"next/\", found \"{}\"",
        relative.display()
      ));
    }

    Ok(if is_from_request {
      format!("from {}", json_stringify(&relative))
    } else {
      format!("import {}", json_stringify(&relative))
    })
  })
  .map_err(|e| error!(e))?;

  // Verify that at least one import was replaced. It's the case today where
  // every template file has at least one import to update, so this ensures that
  // we don't accidentally remove the import replacement code or use the wrong
  // template file.
  if count == 0 {
    return Err(error!("Invariant: Expected to replace at least one import"));
  }

  // Replace all the template variables with the actual values. If a template
  // variable is missing, throw an error.
  let mut replaced = FxIndexSet::default();
  for (key, replacement) in &replacements {
    let full = format!("'{}'", key);

    if content.contains(&full) {
      replaced.insert(*key);
      content = content.replace(&full, &json_stringify(&replacement));
    }
  }

  // Check to see if there's any remaining template variables.
  let regex = lazy_regex::regex!("/VAR_[A-Z_]+");
  let matches = regex
    .find_iter(&content)
    .map(|m| m.as_str().to_string())
    .collect::<Vec<_>>();

  if !matches.is_empty() {
    return Err(error!(
      "Invariant: Expected to replace all template variables, found {}",
      matches.join(", "),
    ));
  }

  // Check to see if any template variable was provided but not used.
  if replaced.len() != replacements.len() {
    // Find the difference between the provided replacements and the replaced
    // template variables. This will let us notify the user of any template
    // variables that were not used but were provided.
    let difference = replacements
      .keys()
      .filter(|k| !replaced.contains(*k))
      .cloned()
      .collect::<Vec<_>>();

    return Err(error!(
      "Invariant: Expected to replace all template variables, missing {} in template",
      difference.join(", "),
    ));
  }

  // Replace the injections.
  let mut injected = FxIndexSet::default();
  for (key, injection) in &injections {
    let full = format!("// INJECT:{}", key);

    if content.contains(&full) {
      // Track all the injections to ensure that we're not missing any.
      injected.insert(*key);
      content = content.replace(&full, &format!("const {} = {}", key, injection));
    }
  }

  // Check to see if there's any remaining injections.
  let regex = lazy_regex::regex!("// INJECT:[A-Za-z0-9_]+");
  let matches = regex
    .find_iter(&content)
    .map(|m| m.as_str().to_string())
    .collect::<Vec<_>>();

  if !matches.is_empty() {
    return Err(error!(
      "Invariant: Expected to inject all injections, found {}",
      matches.join(", "),
    ));
  }

  // Check to see if any injection was provided but not used.
  if injected.len() != injections.len() {
    // Find the difference between the provided replacements and the replaced
    // template variables. This will let us notify the user of any template
    // variables that were not used but were provided.
    let difference = injections
      .keys()
      .filter(|k| !injected.contains(*k))
      .cloned()
      .collect::<Vec<_>>();

    return Err(error!(
      "Invariant: Expected to inject all injections, missing {} in template",
      difference.join(", "),
    ));
  }

  // Replace the optional imports.
  let mut imports_added = FxIndexSet::default();
  for (key, import_path) in &imports {
    let mut full = format!("// OPTIONAL_IMPORT:{}", key);
    let namespace = if !content.contains(&full) {
      full = format!("// OPTIONAL_IMPORT:* as {}", key);
      if content.contains(&full) {
        true
      } else {
        continue;
      }
    } else {
      false
    };

    // Track all the imports to ensure that we're not missing any.
    imports_added.insert(*key);

    if let Some(path) = import_path {
      content = content.replace(
        &full,
        &format!(
          "import {}{} from {}",
          if namespace { "* as " } else { "" },
          key,
          &json_stringify(&path)
        ),
      );
    } else {
      content = content.replace(&full, &format!("const {} = null", key));
    }
  }

  // Check to see if there's any remaining imports.
  let regex = lazy_regex::regex!("// OPTIONAL_IMPORT:(\\* as )?[A-Za-z0-9_]+");
  let matches = regex
    .find_iter(&content)
    .map(|m| m.as_str().to_string())
    .collect::<Vec<_>>();

  if !matches.is_empty() {
    return Err(error!(
      "Invariant: Expected to inject all imports, found {}",
      matches.join(", "),
    ));
  }

  // Check to see if any import was provided but not used.
  if imports_added.len() != imports.len() {
    // Find the difference between the provided imports and the injected
    // imports. This will let us notify the user of any imports that were
    // not used but were provided.
    let difference = imports
      .keys()
      .filter(|k| !imports_added.contains(*k))
      .cloned()
      .collect::<Vec<_>>();

    return Err(error!(
      "Invariant: Expected to inject all imports, missing {} in template",
      difference.join(", "),
    ));
  }

  // Ensure that the last line is a newline.
  if !content.ends_with('\n') {
    content.push('\n');
  }

  Ok(content)
}

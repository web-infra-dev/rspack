#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use crate::{PackageJson, package_json::ParseError};

  #[tokio::test]
  async fn test_json_with_bom() {
    let mock_path = PathBuf::from("package.json");
    let json_with_bom = b"\xEF\xBB\xBF{\"name\": \"example-package\"}".to_vec();

    let result = PackageJson::parse(mock_path.clone(), mock_path, json_with_bom).err();

    assert_eq!(
      result,
      Some(ParseError {
        message: "BOM character found".to_string(),
        index: 0
      })
    );
  }

  #[tokio::test]
  async fn test_normal_json() {
    let mock_path = PathBuf::from("package.json");
    let json_with_bom = r##"{"name": "example-package"}"##.as_bytes().to_vec();

    let parsed = PackageJson::parse(mock_path.clone(), mock_path, json_with_bom).unwrap();

    assert_eq!(parsed.name.unwrap(), "example-package");
  }

  #[tokio::test]
  async fn test_broken_json() {
    let mock_path = PathBuf::from("package.json");
    let json_with_bom = r##"{"broken":"string"##.as_bytes().to_vec();

    let parsed_err = PackageJson::parse(mock_path.clone(), mock_path, json_with_bom).err();

    assert_eq!(
      parsed_err,
      Some(ParseError {
        message: "syntax".to_string(),
        // SIMD error message does not provide the accurate index
        index: 0
      })
    );
  }

  #[tokio::test]
  async fn test_empty_string() {
    let mock_path = PathBuf::from("package.json");
    let json_with_bom = "    ".as_bytes().to_vec();

    let parse_error = PackageJson::parse(mock_path.clone(), mock_path, json_with_bom)
      .err()
      .unwrap();

    assert_eq!(
      parse_error,
      ParseError {
        message: "eof".to_string(),
        index: 0,
      }
    );
  }
}

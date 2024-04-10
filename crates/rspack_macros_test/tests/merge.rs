use rspack_macros::MergeFrom;
use rspack_util::MergeFrom;

mod enum_fields {
  use super::*;

  #[derive(Debug, Clone, PartialEq, Eq, MergeFrom)]
  enum Test {
    A {
      one: String,
      two: String,
      three: String,
    },
    B(String, String),
    C,
  }

  #[test]
  fn test() {
    let t1 = Test::A {
      one: "one".to_string(),
      two: "two".to_string(),
      three: "three".to_string(),
    };
    let t2 = Test::C;
    let t3 = MergeFrom::merge_from(t1.clone(), &t2);
    assert_eq!(t3, t2);
    let t4 = Test::A {
      one: "1".to_string(),
      two: "2".to_string(),
      three: "3".to_string(),
    };
    let t5 = MergeFrom::merge_from(t1.clone(), &t4);
    assert!(matches!(t5, Test::A { one, two, three } if one == "1" && two == "2" && three == "3"));
  }
}

mod enum_base {

  use super::*;

  #[derive(Debug, Clone, PartialEq, Eq, MergeFrom)]
  #[merge_from(enum_base)]
  enum Test {
    A {
      one: String,
      two: String,
      three: String,
    },
    C,
  }

  #[test]
  fn test() {
    let t1 = Test::A {
      one: "one".to_string(),
      two: "two".to_string(),
      three: "three".to_string(),
    };
    let t2 = Test::C;
    let t3 = MergeFrom::merge_from(t1.clone(), &t2);
    assert!(
      matches!(t3, Test::A { one, two, three } if one == "one" && two == "two" && three == "three")
    );
  }
}

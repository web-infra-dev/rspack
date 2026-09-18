use crossbeam_utils::atomic::AtomicCell;
use rspack_core::{BuildMeta, BuildMetaDefaultObject, BuildMetaExportsType};
use rspack_hash::{HashFunction, RspackHash, RspackHasher};

fn hash(value: &impl RspackHash) -> u64 {
  let mut state = RspackHasher::new(&HashFunction::Xxhash64);
  value.hash(&mut state);
  state.finish()
}

#[test]
fn atomic_cell_hashes_like_its_value() {
  #[derive(RspackHash)]
  struct AtomicFields {
    value: AtomicCell<u32>,
  }
  #[derive(RspackHash)]
  struct PlainFields {
    value: u32,
  }
  let atomic = AtomicFields {
    value: AtomicCell::new(7),
  };
  assert_eq!(hash(&atomic), hash(&PlainFields { value: 7 }));
  atomic.value.store(19);
  assert_eq!(hash(&atomic), hash(&PlainFields { value: 19 }));
  for value in [None, Some(false), Some(true)] {
    assert_eq!(hash(&AtomicCell::new(value)), hash(&value));
  }
}

#[test]
fn build_meta_preserves_hash_and_json_fields() {
  let mut meta = BuildMeta::default();
  assert_eq!(hash(&meta), hash(&"{exports_type:}"));
  assert_eq!(
    serde_json::to_string(&meta).unwrap(),
    r#"{"exportsType":"unset"}"#
  );

  meta.set_strict_esm_module(false);
  meta.set_has_top_level_await(true);
  meta.set_esm(true);
  meta.set_is_css_module(false);
  meta.set_need_id_in_concatenation(false);
  meta.set_exports(
    BuildMetaExportsType::Default,
    BuildMetaDefaultObject::RedirectWarn,
  );
  meta.set_side_effect_free(false);
  assert_eq!(
    hash(&meta),
    hash(&concat!(
      "{strict_esm_module:false,has_top_level_await:true,esm:true,",
      "is_css_module:false,need_id_in_concatenation:false,",
      "exports_type:default,default_object:redirect-warn,side_effect_free:false}"
    ))
  );
  assert_eq!(
    serde_json::to_string(&meta).unwrap(),
    concat!(
      r#"{"strictEsmModule":false,"hasTopLevelAwait":true,"esm":true,"#,
      r#""isCssModule":false,"needIdInConcatenation":false,"#,
      r#""exportsType":"default","defaultObject":"redirect-warn","sideEffectFree":false}"#
    )
  );
}

#[test]
fn build_meta_clone_and_cache_restore_have_independent_cells() {
  for side_effect_free in [None, Some(false), Some(true)] {
    let mut meta = BuildMeta::default()
      .with_exports_type(BuildMetaExportsType::Default)
      .with_default_object(BuildMetaDefaultObject::RedirectWarn);
    if let Some(value) = side_effect_free {
      meta.set_side_effect_free(value);
    }
    let mut expected = serde_json::json!({
      "exportsType": "default",
      "defaultObject": "redirect-warn"
    });
    if let Some(value) = side_effect_free {
      expected["sideEffectFree"] = value.into();
    }
    assert_eq!(serde_json::to_value(&meta).unwrap(), expected);
    let cloned = meta.clone();
    let bytes = rspack_cacheable::to_bytes(&meta, &()).unwrap();
    let restored: BuildMeta = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
    assert_eq!(hash(&restored), hash(&meta));
    assert_eq!(
      serde_json::to_value(&restored).unwrap(),
      serde_json::to_value(&meta).unwrap()
    );

    meta.set_exports(
      BuildMetaExportsType::Namespace,
      BuildMetaDefaultObject::False,
    );
    meta.set_side_effect_free(true);
    for independent in [cloned, restored] {
      assert_eq!(independent.exports_type(), BuildMetaExportsType::Default);
      assert!(matches!(
        independent.default_object(),
        BuildMetaDefaultObject::RedirectWarn
      ));
      assert_eq!(independent.side_effect_free_option(), side_effect_free);
    }
  }
}

use super::OverlayValue;
use crate::ModuleId;

#[derive(Debug, Clone)]
pub struct DenseModuleIdOverlayMap<V> {
  base: Vec<Option<V>>,
  overlay: Option<Vec<Option<OverlayValue<V>>>>,
}

impl<V> Default for DenseModuleIdOverlayMap<V> {
  fn default() -> Self {
    Self {
      base: Vec::new(),
      overlay: None,
    }
  }
}

impl<V> DenseModuleIdOverlayMap<V> {
  #[inline]
  pub fn checkpoint(&mut self) {
    self.overlay.get_or_insert_with(Vec::new);
  }

  #[inline]
  pub fn reset(&mut self) {
    self.overlay = None;
  }

  #[inline]
  pub fn insert(&mut self, key: &ModuleId, value: V) {
    let index = module_id_to_index(key);
    if self.overlay.is_some() {
      Self::ensure_len(self.overlay(), index);
      self.overlay.as_mut().expect("overlay checked above")[index] =
        Some(OverlayValue::Value(value));
    } else {
      Self::ensure_len(&mut self.base, index);
      self.base[index] = Some(value);
    }
  }

  #[inline]
  pub fn remove(&mut self, key: &ModuleId) {
    let index = module_id_to_index(key);
    if self.overlay.is_some() {
      Self::ensure_len(self.overlay(), index);
      self.overlay.as_mut().expect("overlay checked above")[index] = Some(OverlayValue::Tombstone);
    } else if let Some(value) = self.base.get_mut(index) {
      *value = None;
    }
  }

  #[inline]
  pub fn get(&self, key: &ModuleId) -> Option<&V> {
    let index = module_id_to_index(key);
    if let Some(overlay) = &self.overlay
      && let Some(Some(value)) = overlay.get(index)
    {
      return match value {
        OverlayValue::Value(value) => Some(value),
        OverlayValue::Tombstone => None,
      };
    }
    self.base.get(index).and_then(Option::as_ref)
  }

  #[inline]
  pub fn get_mut(&mut self, key: &ModuleId) -> Option<&mut V>
  where
    V: Clone,
  {
    let index = module_id_to_index(key);
    if self.overlay.is_some() {
      self.materialize_overlay_value(index);
      let overlay = self.overlay.as_mut().expect("overlay checked above");
      match overlay.get_mut(index).and_then(Option::as_mut) {
        Some(OverlayValue::Value(value)) => Some(value),
        _ => None,
      }
    } else {
      self.base.get_mut(index).and_then(Option::as_mut)
    }
  }

  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = (ModuleId, &V)> {
    let overlay = self.overlay.as_ref();
    let base_len = self.base.len();
    let base_iter = self
      .base
      .iter()
      .enumerate()
      .filter_map(move |(index, value)| {
        if let Some(Some(overlay_value)) = overlay.and_then(|overlay| overlay.get(index)) {
          return match overlay_value {
            OverlayValue::Value(value) => Some((ModuleId::from(index as u32), value)),
            OverlayValue::Tombstone => None,
          };
        }
        value
          .as_ref()
          .map(|value| (ModuleId::from(index as u32), value))
      });
    let overlay_iter = self
      .overlay
      .as_ref()
      .into_iter()
      .flat_map(move |overlay| overlay.iter().enumerate().skip(base_len))
      .filter_map(|(index, value)| match value.as_ref()? {
        OverlayValue::Value(value) => Some((ModuleId::from(index as u32), value)),
        OverlayValue::Tombstone => None,
      });
    base_iter.chain(overlay_iter)
  }

  #[inline]
  fn materialize_overlay_value(&mut self, index: usize)
  where
    V: Clone,
  {
    let overlay = self.overlay.as_ref().expect("overlay checked above");
    if matches!(overlay.get(index), Some(Some(_))) {
      return;
    }

    if let Some(value) = self.base.get(index).and_then(Option::as_ref).cloned() {
      Self::ensure_len(self.overlay(), index);
      self.overlay.as_mut().expect("overlay checked above")[index] =
        Some(OverlayValue::Value(value));
    }
  }

  #[inline]
  fn overlay(&mut self) -> &mut Vec<Option<OverlayValue<V>>> {
    self.overlay.get_or_insert_with(Vec::new)
  }

  #[inline]
  fn ensure_len<T>(values: &mut Vec<Option<T>>, index: usize) {
    if values.len() <= index {
      values.resize_with(index + 1, || None);
    }
  }
}

#[inline]
fn module_id_to_index(id: &ModuleId) -> usize {
  id.as_number()
    .expect("dense module ids should always be numeric") as usize
}

#[cfg(test)]
mod tests {
  use crate::{ModuleId, module_graph::rollback::DenseModuleIdOverlayMap};

  #[test]
  fn checkpoint_inserts_apply_only_to_overlay() {
    let mut map = DenseModuleIdOverlayMap::default();
    let a = ModuleId::from(0);
    let b = ModuleId::from(1);

    map.insert(&a, 1);
    map.checkpoint();
    map.insert(&b, 2);
    map.insert(&a, 3);

    assert_eq!(map.get(&a), Some(&3));
    assert_eq!(map.get(&b), Some(&2));
    assert_eq!(
      map.iter().collect::<Vec<_>>(),
      vec![(a.clone(), &3), (b, &2)]
    );

    map.reset();

    assert_eq!(map.get(&a), Some(&1));
  }

  #[test]
  fn remove_in_overlay_masks_base() {
    let mut map = DenseModuleIdOverlayMap::default();
    let a = ModuleId::from(0);

    map.insert(&a, 1);
    map.checkpoint();
    map.remove(&a);

    assert_eq!(map.get(&a), None);
    assert_eq!(
      map.iter().collect::<Vec<_>>(),
      Vec::<(ModuleId, &i32)>::new()
    );

    map.reset();

    assert_eq!(map.get(&a), Some(&1));
  }
}

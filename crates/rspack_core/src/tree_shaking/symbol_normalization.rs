use std::{borrow::BorrowMut, collections::VecDeque, path::PathBuf, str::FromStr};

use hashbrown::{hash_map::Entry, hash_set, HashMap, HashSet};
use petgraph::prelude::GraphMap;
use rspack_error::Error;
use rspack_symbol::{IndirectTopLevelSymbol, Symbol};
use sugar_path::SugarPath;
use swc_atoms::JsWord;
use ustr::Ustr;

use crate::join_string_component;

use super::{
  visitor::{ModuleRefAnalyze, SymbolRef, TreeShakingResult},
  BailoutReason,
};

pub struct SymbolRefNormalization<'a> {
  analyze_results: HashMap<Ustr, TreeShakingResult>,
  entry_module_identifiers: &'a HashSet<String>,
}

impl<'a> SymbolRefNormalization<'a> {
  fn create_inherit_graph(&self) -> GraphMap<&Ustr, (), petgraph::Directed> {
    let mut g = petgraph::graphmap::DiGraphMap::new();
    for (module_id, result) in self.analyze_results.iter() {
      for export_all_module_id in result.inherit_export_maps.keys() {
        g.add_edge(module_id, export_all_module_id, ());
      }
    }
    g
  }

  fn collect_reachable_symbol(
    &self,
    entry_identifier: Ustr,
    used_indirect_symbol: &mut HashSet<IndirectTopLevelSymbol>,
    bailout_module_identifiers: &HashMap<Ustr, BailoutReason>,
    evaluated_module_identifiers: &mut HashSet<Ustr>,
    used_export_module_identifiers: &mut HashSet<Ustr>,
    errors: &mut Vec<Error>,
  ) -> HashSet<Symbol> {
    let mut used_symbol_set = HashSet::new();
    let mut q = VecDeque::new();
    let entry_module_result = match self.analyze_results.get(&entry_identifier) {
      Some(result) => result,
      None => {
        panic!("Can't get analyze result from entry_identifier");
      }
    };

    // deduplicate reexport in entry module start
    let mut export_symbol_count_map: HashMap<JsWord, (SymbolRef, usize)> = entry_module_result
      .export_map
      .iter()
      .map(|(symbol_name, symbol_ref)| (symbol_name.clone(), (symbol_ref.clone(), 1)))
      .collect();
    // All the reexport star symbol should be included in the bundle
    // TODO: esbuild will hidden the duplicate reexport, webpack will emit an error
    // which should we align to?
    for (_, inherit_map) in entry_module_result.inherit_export_maps.iter() {
      for (atom, symbol_ref) in inherit_map.iter() {
        match export_symbol_count_map.entry(atom.clone()) {
          Entry::Occupied(mut occ) => {
            occ.borrow_mut().get_mut().1 += 1;
          }
          Entry::Vacant(vac) => {
            vac.insert((symbol_ref.clone(), 1));
          }
        };
      }
    }

    q.extend(export_symbol_count_map.into_iter().filter_map(|(_, v)| {
      if v.1 == 1 {
        Some(v.0)
      } else {
        None
      }
    }));
    // deduplicate reexport in entry end

    for item in entry_module_result.export_map.values() {
      mark_symbol(
        item.clone(),
        &mut used_symbol_set,
        used_indirect_symbol,
        analyze_map,
        &mut q,
        bailout_module_identifiers,
        evaluated_module_identifiers,
        used_export_module_identifiers,
        errors,
      );
    }

    while let Some(sym_ref) = q.pop_front() {
      mark_symbol(
        sym_ref,
        &mut used_symbol_set,
        used_indirect_symbol,
        analyze_map,
        &mut q,
        bailout_module_identifiers,
        evaluated_module_identifiers,
        used_export_module_identifiers,
        errors,
      );
    }
    used_symbol_set
  }

  fn get_reachable(start: Ustr, g: &GraphMap<&Ustr, (), petgraph::Directed>) -> HashSet<Ustr> {
    let mut visited: HashSet<Ustr> = HashSet::new();
    let mut reachable_module_id = HashSet::new();
    let mut q = VecDeque::from_iter([start]);
    while let Some(cur) = q.pop_front() {
      match visited.entry(cur) {
        hashbrown::hash_set::Entry::Occupied(_) => continue,
        hashbrown::hash_set::Entry::Vacant(vac) => vac.insert(),
      }
      if cur != start {
        reachable_module_id.insert(cur);
      }
      q.extend(g.neighbors_directed(&cur, petgraph::Direction::Outgoing));
    }
    reachable_module_id
  }

  pub(crate) fn normalize(&mut self) {
    let mut used_symbol_ref: HashSet<SymbolRef> = HashSet::default();
    let mut bail_out_module_identifiers = HashMap::default();
    let mut evaluated_module_identifiers = HashSet::new();
    for analyze_result in self.analyze_results.values() {
      let forced_side_effects = self
        .entry_module_identifiers
        .contains(analyze_result.module_identifier.as_str());
      // side_effects: true
      if forced_side_effects || !analyze_result.side_effects_free {
        evaluated_module_identifiers.insert(analyze_result.module_identifier);
        used_symbol_ref.extend(analyze_result.used_symbol_ref.iter().cloned());
      }
      bail_out_module_identifiers.extend(analyze_result.bail_out_module_identifiers.clone());
    }

    // calculate relation of module that has `export * from 'xxxx'`
    let inherit_export_ref_graph = self.create_inherit_graph();
    // key is the module_id of module that potential have reexport all symbol from other module
    // value is the set which contains several module_id the key related module need to inherit
    let map_of_inherit_map = get_extends_map(&inherit_export_ref_graph);

    for (module_id, inherit_export_module_id) in map_of_inherit_map.iter() {
      // This is just a work around for rustc checker, because we have immutable and mutable borrow at the same time.
      let mut inherit_export_maps = {
        let main_module = self.analyze_results.get_mut(module_id).unwrap();
        std::mem::take(&mut main_module.inherit_export_maps)
      };
      for inherit_export_module_identifier in inherit_export_module_id {
        let export_module = self
          .analyze_results
          .get(inherit_export_module_identifier)
          .unwrap()
          .export_map
          .iter()
          .filter_map(|(k, v)| {
            // export * should not reexport default export
            if k == "default" {
              None
            } else {
              Some((k.clone(), v.clone()))
            }
          })
          .collect::<HashMap<JsWord, SymbolRef>>();
        inherit_export_maps.insert(*inherit_export_module_identifier, export_module);
      }
      self
        .analyze_results
        .get_mut(module_id)
        .unwrap()
        .inherit_export_maps = inherit_export_maps;
    }
    let mut errors = vec![];
    let mut used_symbol = HashSet::new();

    let mut used_export_module_identifiers: HashSet<Ustr> = HashSet::new();
    // Marking used symbol and all reachable export symbol from the used symbol for each module
    let used_symbol_from_import = mark_used_symbol_with(
      &analyze_results,
      VecDeque::from_iter(used_symbol_ref.into_iter()),
      &bail_out_module_identifiers,
      &mut evaluated_module_identifiers,
      &mut used_indirect_symbol,
      &mut used_export_module_identifiers,
      &mut errors,
    );

    used_symbol.extend(used_symbol_from_import);

    // We considering all export symbol in each entry module as used for now
    for entry in self.entry_modules() {
      let used_symbol_set = collect_reachable_symbol(
        &analyze_results,
        ustr(&entry),
        &mut used_indirect_symbol,
        &bail_out_module_identifiers,
        &mut evaluated_module_identifiers,
        &mut used_export_module_identifiers,
        &mut errors,
      );
      used_symbol.extend(used_symbol_set);
    }

    // TODO: SideEffects: only

    for result in self.analyze_results.values() {
      if !bail_out_module_identifiers.contains_key(&result.module_identifier)
        && !self
          .entry_module_identifiers
          .contains(result.module_identifier.as_str())
        && result.side_effects_free
        && !used_export_module_identifiers.contains(&result.module_identifier)
        && result.inherit_export_maps.is_empty()
      {
        dbg!(&result.module_identifier);
        self
          .module_graph
          .module_graph_module_by_identifier_mut(result.module_identifier.as_str())
          .unwrap()
          .used = false;
      }
    }
    dbg!(&used_symbol, &used_indirect_symbol);
  }
}

fn mark_used_symbol_with(
  analyze_map: &hashbrown::HashMap<Ustr, TreeShakingResult>,
  mut init_queue: VecDeque<SymbolRef>,
  bailout_module_identifiers: &HashMap<Ustr, BailoutReason>,
  evaluated_module_identifiers: &mut HashSet<Ustr>,
  used_indirect_symbol_set: &mut HashSet<IndirectTopLevelSymbol>,
  used_export_module_identifiers: &mut HashSet<Ustr>,
  errors: &mut Vec<Error>,
) -> HashSet<Symbol> {
  let mut used_symbol_set = HashSet::new();
  let mut visited = HashSet::new();

  while let Some(sym_ref) = init_queue.pop_front() {
    if visited.contains(&sym_ref) {
      continue;
    } else {
      visited.insert(sym_ref.clone());
    }
    mark_symbol(
      sym_ref,
      &mut used_symbol_set,
      used_indirect_symbol_set,
      analyze_map,
      &mut init_queue,
      bailout_module_identifiers,
      evaluated_module_identifiers,
      used_export_module_identifiers,
      errors,
    );
  }
  used_symbol_set
}

fn mark_symbol(
  symbol_ref: SymbolRef,
  used_symbol_set: &mut HashSet<Symbol>,
  used_indirect_symbol_set: &mut HashSet<IndirectTopLevelSymbol>,
  analyze_map: &HashMap<Ustr, TreeShakingResult>,
  q: &mut VecDeque<SymbolRef>,
  bailout_module_identifiers: &HashMap<Ustr, BailoutReason>,
  evaluated_module_identifiers: &mut HashSet<Ustr>,
  used_export_module_identifiers: &mut HashSet<Ustr>,
  errors: &mut Vec<Error>,
) {
  // We don't need mark the symbol usage if it is from a bailout module because
  // bailout module will skipping tree-shaking anyway
  // if debug_care_module_id(symbol_ref.module_identifier()) {
  // }
  let is_bailout_module_identifier =
    bailout_module_identifiers.contains_key(&symbol_ref.module_identifier());
  match &symbol_ref {
    SymbolRef::Direct(symbol) => {
      used_export_module_identifiers.insert(symbol.uri());
    }
    SymbolRef::Indirect(indirect) => {
      used_export_module_identifiers.insert(indirect.uri());
    }
    SymbolRef::Star(_) => {}
  };
  match symbol_ref {
    SymbolRef::Direct(symbol) => match used_symbol_set.entry(symbol) {
      hash_set::Entry::Occupied(_) => {}
      hash_set::Entry::Vacant(vac) => {
        let module_result = analyze_map.get(&vac.get().uri()).unwrap();
        if let Some(set) = module_result
          .reachable_import_of_export
          .get(&vac.get().id().atom)
        {
          q.extend(set.iter().cloned());
        };
        // Assume the module name is app.js
        // ```js
        // import {myanswer, secret} from './lib'
        // export {myanswer as m, secret as s}
        // ```
        // In such scenario there are two `myanswer` binding would create
        // one for `app.js`, one for `lib.js`
        // the binding in `app.js` used for shake the `export {xxx}`
        // In other words, we need two binding for supporting indirect redirect.
        if let Some(symbol_ref) = module_result.import_map.get(vac.get().id()) {
          q.push_back(symbol_ref.clone());
        }
        if !evaluated_module_identifiers.contains(&vac.get().uri()) {
          evaluated_module_identifiers.insert(vac.get().uri());
          q.extend(module_result.used_symbol_ref.clone());
        }
        vac.insert();
      }
    },
    SymbolRef::Indirect(indirect_symbol) => {
      used_indirect_symbol_set.insert(indirect_symbol.clone());
      let module_result = match analyze_map.get(&indirect_symbol.uri) {
        Some(module_result) => module_result,
        None => {
          // eprintln!(
          //   "Can't get optimize dep result for module {}",
          //   indirect_symbol.uri,
          // );
          return;
        }
      };
      let symbol = match module_result.export_map.get(&indirect_symbol.id) {
        Some(symbol) => symbol.clone(),
        None => {
          // TODO: better diagnostic and handle if multiple extends_map has export same symbol
          let mut ret = vec![];
          // Checking if any inherit export map is belong to a bailout module
          let mut has_bailout_module_identifiers = false;
          for (module_identifier, extends_export_map) in module_result.inherit_export_maps.iter() {
            if let Some(value) = extends_export_map.get(&indirect_symbol.id) {
              ret.push((module_identifier, value));
            }
            has_bailout_module_identifiers = has_bailout_module_identifiers
              || bailout_module_identifiers.contains_key(module_identifier);
          }
          match ret.len() {
            0 => {
              // TODO: Better diagnostic handle if source module does not have the export
              // let map = analyze_map.get(&module_result.module_identifier).unwrap();
              // dbg!(&map);
              if !is_bailout_module_identifier && !has_bailout_module_identifiers {
                eprint!(
                  "{} did not export `{}`, imported by {}",
                  module_result.module_identifier,
                  indirect_symbol.id,
                  indirect_symbol.importer()
                );
                return;
              } else {
                // TODO: This branch should be remove after we analyze module.exports
                // If one of inherit module is a bailout module, that most probably means that module has some common js export
                // which we don't analyze yet, we just pass it. It is alright because we don't modified the ast of bailout module
                return;
              }
            }
            1 => ret[0].1.clone(),
            // multiple export candidate in reexport
            // mark the first symbol_ref as used, align to webpack
            _ => {
              // TODO: better traceable diagnostic
              let mut error_message = format!(
                "Conflicting star exports for the name '{}' in ",
                indirect_symbol.id
              );
              let cwd = std::env::current_dir();
              let module_identifier_list = ret
                .iter()
                .map(|(module_identifier, _)| {
                  // try to use relative path which should have better DX
                  match cwd {
                    Ok(ref cwd) => {
                      let p = PathBuf::from_str(module_identifier.as_str()).unwrap();
                      p.relative(cwd.as_path())
                        .as_path()
                        .to_string_lossy()
                        .to_string()
                    }
                    // if we can't get the cwd, fallback to module identifier
                    Err(_) => module_identifier.to_string(),
                  }
                })
                .collect::<Vec<_>>();
              error_message += &join_string_component(module_identifier_list);
              errors.push(Error::InternalError(error_message));
              ret[0].1.clone()
            }
          }
        }
      };
      q.push_back(symbol);
    }
    SymbolRef::Star(star) => {
      // If a star ref is used. e.g.
      // ```js
      // import * as all from './test.js'
      // all
      // ```
      // then, all the exports in `test.js` including
      // export defined in `test.js` and all realted
      // reexport should be marked as used
      let module_result = analyze_map.get(&star).unwrap();
      for symbol_ref in module_result.export_map.values() {
        q.push_back(symbol_ref.clone());
      }

      for (_, extend_map) in module_result.inherit_export_maps.iter() {
        q.extend(extend_map.values().cloned());
      }
    }
  }
}

fn get_extends_map(
  export_all_ref_graph: &GraphMap<&Ustr, (), petgraph::Directed>,
) -> HashMap<Ustr, HashSet<Ustr>> {
  let mut map = HashMap::new();
  for node in export_all_ref_graph.nodes() {
    let reachable_set = get_reachable(*node, export_all_ref_graph);
    map.insert(*node, reachable_set);
  }
  map
}

use smol_str::SmolStr;


#[derive(Debug, Default)]
pub struct Chunk {
  // pub id: SmolStr,
  pub module_ids: Vec<SmolStr>,
}
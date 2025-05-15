use browserslist::{resolve, Distrib, Error, Opts};

pub fn resolve_browserslist(opts: &Opts, browserslist: &str) -> Result<Vec<Distrib>, Error> {
  resolve(opts, browserslist)
}

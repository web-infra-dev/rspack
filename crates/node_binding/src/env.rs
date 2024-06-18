use std::env;
use std::ffi::OsStr;

pub fn set_env_if_unset<K, V>(k: K, v: V)
where
  K: AsRef<OsStr>,
  V: AsRef<OsStr>,
{
  match env::var(&k) {
    Ok(_) => (),
    Err(_) => {
      env::set_var(&k, v);
    }
  }
}

export function helperPure() {
  return 1;
}

export function pureUsesHelper() {
  return helperPure();
}

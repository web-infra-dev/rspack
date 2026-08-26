function helperPure() {
  return 1;
}

export function impureShadowedPureCall(helperPure) {
  return helperPure();
}

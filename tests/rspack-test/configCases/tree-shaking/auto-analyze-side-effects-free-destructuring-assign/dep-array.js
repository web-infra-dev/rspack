function arrayFn() {
  return "pure";
}

[arrayFn] = [() => {
  globalThis.sideEffectCount += 1;
  return "array";
}];

export { arrayFn };

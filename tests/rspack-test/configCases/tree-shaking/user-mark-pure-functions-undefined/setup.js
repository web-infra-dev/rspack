globalThis.sideEffectCount = 0;
globalThis.notExistFunction = () => {
  globalThis.sideEffectCount += 1;
};

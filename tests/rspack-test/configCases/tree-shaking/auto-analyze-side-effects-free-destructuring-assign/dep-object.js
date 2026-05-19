function objectFn() {
  return "pure";
}

({ objectFn } = {
  objectFn: () => {
    globalThis.sideEffectCount += 1;
    return "object";
  }
});

export { objectFn };

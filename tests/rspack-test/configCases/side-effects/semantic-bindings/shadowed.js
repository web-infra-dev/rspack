globalThis.__semantic_shadowed_calls__ = [];

function selected() {
  return 1;
}

function invoke(selected) {
  selected();
}

// The callee inside invoke resolves to its parameter, not the pure top-level function.
export const unused = invoke(() => globalThis.__semantic_shadowed_calls__.push('parameter'));

{
  const selected = () => globalThis.__semantic_shadowed_calls__.push('block');
  selected();
}

export { selected };

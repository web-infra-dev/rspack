globalThis.__semantic_duplicate_calls__ = 0;

function duplicate() {
  return 1;
}

// Both declarations belong to one semantic symbol; the first body is not sufficient.
function duplicate() {
  globalThis.__semantic_duplicate_calls__++;
}

duplicate();

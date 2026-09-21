// Internal synchronous binding protocol: fill or patch a target using source
// indices. Do not retain arguments or invoke callbacks; indices are call-scoped.
export function applyIndexedArrayUpdates<T>(
  source: readonly T[],
  target: T[],
  commands: Uint32Array,
): void {
  const mode = commands[0];
  const length = commands[1];
  if (mode === 0) {
    // Preserve the target's identity without repeatedly growing its storage.
    target.length = length;
    for (let i = 0, cursor = 2; cursor < commands.length; i++, cursor++) {
      target[i] = source[commands[cursor]];
    }
  } else {
    for (let cursor = 2; cursor < commands.length; cursor += 2) {
      target[commands[cursor]] = source[commands[cursor + 1]];
    }
  }
  target.length = length;
}

// Remove distinct indices in descending order by moving the last element into
// each gap. This does not preserve order. Do not retain arguments or invoke callbacks.
export function swapRemoveArrayElements<T>(
  array: T[],
  removedIndices: Uint32Array,
): void {
  let length = array.length;
  for (const index of removedIndices) {
    length--;
    if (index !== length) array[index] = array[length];
  }
  array.length = length;
}

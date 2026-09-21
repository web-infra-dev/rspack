// Public iterators share one immutable snapshot until the source array changes.
// Weak keys let snapshots disappear with their source unless a caller retains them.
const arraySnapshots = new WeakMap<readonly unknown[], readonly unknown[]>();

export function getArraySnapshot<T>(array: readonly T[]): readonly T[] {
  let snapshot = arraySnapshots.get(array) as readonly T[] | undefined;
  if (snapshot === undefined) {
    snapshot = array.slice();
    arraySnapshots.set(array, snapshot);
  }
  return snapshot;
}

// Internal synchronous binding protocol: fill or patch targets using source
// indices. Do not retain arguments or invoke callbacks; indices are call-scoped.
export function applyIndexedArrayUpdates<T>(
  source: readonly T[],
  targets: T[][],
  commands: Uint32Array,
): void {
  let cursor = 0;
  for (let targetIndex = 0; targetIndex < targets.length; targetIndex++) {
    const target = targets[targetIndex];
    // Invalidate before writing, including if an update throws partway through.
    arraySnapshots.delete(target);
    const mode = commands[cursor++];
    const length = commands[cursor++];
    const payloadLength = commands[cursor++];
    const end = cursor + payloadLength;
    if (mode === 0) {
      // Preserve the target's identity without repeatedly growing its storage.
      target.length = length;
      for (let i = 0; cursor < end; i++, cursor++) {
        target[i] = source[commands[cursor]];
      }
    } else {
      for (; cursor < end; cursor += 2) {
        target[commands[cursor]] = source[commands[cursor + 1]];
      }
    }
    target.length = length;
  }
}

// Remove distinct indices in descending order by moving the last element into
// each gap. This does not preserve order. Do not retain arguments or invoke callbacks.
export function swapRemoveArrayElements<T>(
  array: T[],
  removedIndices: Uint32Array,
): void {
  if (removedIndices.length !== 0) arraySnapshots.delete(array);
  let length = array.length;
  for (const index of removedIndices) {
    length--;
    if (index !== length) array[index] = array[length];
  }
  array.length = length;
}

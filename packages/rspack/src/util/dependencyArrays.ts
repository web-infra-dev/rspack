// Internal synchronous binding protocol. The helper must not capture a compiler,
// retain its arguments, or invoke callbacks: pool indices expire after this call.
export function materializeDependencyArrays(
  strings: readonly string[],
  targets: string[][],
  commands: Uint32Array,
): void {
  let cursor = 0;
  for (let targetIndex = 0; targetIndex < targets.length; targetIndex++) {
    const target = targets[targetIndex];
    const mode = commands[cursor++];
    const length = commands[cursor++];
    const payloadLength = commands[cursor++];
    const end = cursor + payloadLength;
    if (mode === 0) {
      // Preserve the target's identity without repeatedly growing its storage.
      target.length = length;
      for (let i = 0; cursor < end; i++, cursor++) {
        target[i] = strings[commands[cursor]];
      }
    } else {
      for (; cursor < end; cursor += 2) {
        target[commands[cursor]] = strings[commands[cursor + 1]];
      }
    }
    target.length = length;
  }
}

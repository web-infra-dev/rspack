export function assertNotNill(value: unknown): asserts value {
  if (value == null) {
    throw Error(`${value} should not be undefined or null`);
  }
}

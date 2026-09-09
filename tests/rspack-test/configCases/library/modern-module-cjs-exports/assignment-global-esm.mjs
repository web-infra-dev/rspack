export const before = exports;
exports = { value: 1 };
export const after = exports;

export function assign(value) {
  exports = value;
  return exports;
}

export function destructure(value) {
  ({ exports } = value);
  return exports;
}

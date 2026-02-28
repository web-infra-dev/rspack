// @ts-nocheck
export function toBeTypeOf(received, expected) {
  const objType = typeof received;
  const pass = objType === expected;

  const message = pass
    ? () =>
        `${this.utils.matcherHint('.not.toBeTypeOf')}\n\nExpected value to not be (using typeof):\n  ${this.utils.printExpected(expected)}\nReceived:\n  ${this.utils.printReceived(objType)}`
    : () =>
        `${this.utils.matcherHint('.toBeTypeOf')}\n\nExpected value to be (using typeof):\n  ${this.utils.printExpected(expected)}\nReceived:\n  ${this.utils.printReceived(objType)}`;

  return { message, pass };
}

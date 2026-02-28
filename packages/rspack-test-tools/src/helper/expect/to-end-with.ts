// @ts-nocheck
export function toEndWith(received, expected) {
  const pass = typeof received === 'string' && received.endsWith(expected);

  const message = pass
    ? () =>
        `${this.utils.matcherHint('.not.toEndWith')}\n\nExpected value to not end with:\n  ${this.utils.printExpected(expected)}\nReceived:\n  ${this.utils.printReceived(received)}`
    : () =>
        `${this.utils.matcherHint('.toEndWith')}\n\nExpected value to end with:\n  ${this.utils.printExpected(expected)}\nReceived:\n  ${this.utils.printReceived(received)}`;

  return { message, pass };
}

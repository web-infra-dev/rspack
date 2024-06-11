// @ts-nocheck
export function toEndWith(received, expected) {
	const pass = typeof received === "string" && received.endsWith(expected);

	const message = pass
		? () =>
				this.utils.matcherHint(".not.toEndWith") +
				"\n\n" +
				"Expected value to not end with:\n" +
				`  ${this.utils.printExpected(expected)}\n` +
				"Received:\n" +
				`  ${this.utils.printReceived(received)}`
		: () =>
				this.utils.matcherHint(".toEndWith") +
				"\n\n" +
				"Expected value to end with:\n" +
				`  ${this.utils.printExpected(expected)}\n` +
				"Received:\n" +
				`  ${this.utils.printReceived(received)}`;

	return { message, pass };
}

const exports = { hiddenReceiverValue: 42 };

export function getCommonJsSyntheticModule(exports) {
	const exportNames = Object.getOwnPropertyNames(exports);
	const namespace =
		exports !== null &&
		(typeof exports === "object" || typeof exports === "function")
			? Object.defineProperties(
					{},
					Object.fromEntries(
						exportNames.map((name) => [
							name,
							{
								enumerable: true,
								get() {
									return exports[name];
								}
							}
						])
					)
				)
			: { default: exports };
	return namespace;
}

export function isFunctionBinding(rspackRequire) {
	return typeof rspackRequire === "function";
}

export { exports };

it("should keep shadowed parameter names and rename top-level exports bindings consistently", () => {
	expect(typeof exports).toBe("object");
	expect(exports).toEqual({ hiddenReceiverValue: 42 });

	const namespace = getCommonJsSyntheticModule({ hiddenReceiverValue: 42 });
	expect(Object.keys(namespace)).toEqual(["hiddenReceiverValue"]);
	expect(namespace.hiddenReceiverValue).toBe(42);

	expect(getCommonJsSyntheticModule(42)).toEqual({ default: 42 });
	expect(isFunctionBinding(() => {})).toBe(true);
});

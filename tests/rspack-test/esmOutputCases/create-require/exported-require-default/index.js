import defaultRequire, {
	aliasedCalleeRequire,
	assignedRequire,
	copiedRequire,
	directRequire,
	exportedRequire
} from "./created-require.js";

export {
	aliasedCalleeRequire,
	assignedRequire,
	copiedRequire,
	defaultRequire,
	directRequire,
	exportedRequire
};

it("keeps an exported created require with default requireResolve in ESM output", () => {
	for (const require of [
		aliasedCalleeRequire,
		assignedRequire,
		copiedRequire,
		defaultRequire,
		directRequire,
		exportedRequire
	]) {
		expect(typeof require).toBe("function");
		expect(require.resolve("path")).toBe("path");
	}
});

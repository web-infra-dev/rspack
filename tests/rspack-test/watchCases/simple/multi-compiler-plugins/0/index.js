const value = require("./a-value");

it("should apply plugins once to the MultiCompiler and fire its hooks", () => {
	expect(value).toBe(`a${WATCH_STEP}`);
	expect(STATE.applied).toBe(1);
	expect(STATE.thisIsCompiler).toBe(true);
	expect(STATE.optionsPlugins).toBe(true);
	// Step 0 builds both children, step 1 only `a`, whose file changed.
	const expected =
		WATCH_STEP === "0" ? ["watchRun a", "watchRun b"] : ["watchRun a"];
	expect(EVENTS.slice(0, -1).sort()).toEqual(expected);
	expect(EVENTS[EVENTS.length - 1]).toBe("done 2");
	EVENTS.length = 0;
});

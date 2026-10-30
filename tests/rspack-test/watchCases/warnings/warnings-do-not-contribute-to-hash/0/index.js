require("./warning-loader.mjs!./changing-file");

it("should keep the hash when only warnings change", function() {
	switch(WATCH_STEP) {
		case "0":
			STATE.hash = __STATS__.hash;
			break;
		case "1":
			// Unlike webpack, warning text does not affect the hash because the output is unchanged.
			expect(__STATS__.hash).toBe(STATE.hash);
			break;
	}
});

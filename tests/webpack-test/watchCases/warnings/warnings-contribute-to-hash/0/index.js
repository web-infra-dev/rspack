require("./warning-loader!./changing-file");

it("should detect a change on warnings change", function() {
	switch(WATCH_STEP) {
		case "0":
			STATE.hash = __STATS__.hash;
			break;
		case "1":
			expect(__STATS__.hash).not.toBe(STATE.hash);
			break;
	}
});

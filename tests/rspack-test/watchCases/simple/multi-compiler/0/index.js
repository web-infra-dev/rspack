require("./changing-file")
it("should watch for changes", function() {
	switch(WATCH_STEP) {
		case "0":
			expect(__STATS__.children).toHaveLength(2);
			break;
		case "1":
			expect(__STATS__.children).toHaveLength(1);
			break;
	}
})

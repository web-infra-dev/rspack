import cm from "./changing-module";
import m from "./module";

it("should flag ES modules correctly", function() {
	expect(m).toBe("module" + WATCH_STEP);
	switch(WATCH_STEP) {
		case "0":
			expect(cm).toBe("original");
			break;
		case "1":
			expect(cm).toBe("change");
			break;
	}
});

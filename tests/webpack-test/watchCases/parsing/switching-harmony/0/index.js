import cc from "./cc";
import ch from "./ch";
import hc from "./hc";
import hh from "./hh";

it("should flag modules correctly", function() {
	expect(hh).toBe("hh" + WATCH_STEP);
	expect(cc).toBe("cc" + WATCH_STEP);
	expect(hc).toBe("hc" + WATCH_STEP);
	expect(ch).toBe("ch" + WATCH_STEP);
	expect(require("./hh").default).toBe("hh" + WATCH_STEP);
	expect(require("./cc")).toBe("cc" + WATCH_STEP);
	switch(WATCH_STEP) {
		case "0":
			expect(require("./hc").default).toBe("hc0");
			expect(require("./ch")).toBe("ch0");
			break;
		case "1":
			expect(require("./hc")).toBe("hc1");
			expect(require("./ch").default).toBe("ch1");
			break;
	}
});

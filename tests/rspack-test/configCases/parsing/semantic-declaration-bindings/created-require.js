import { createRequire } from "module";

it("resets a parameter alias when its var declaration creates a require", () => {
	const outer = createRequire(import.meta.url);
	(function (load) {
		var load = createRequire(import.meta.url);
		expect(load("./value")).toBe("dependency");
		load = request => request;
		expect(load("./missing-local")).toBe("./missing-local");
	})(outer);
	expect(outer("./value")).toBe("dependency");
});

it("resets a parameter alias when its var declaration copies a require", () => {
	const outer = createRequire(import.meta.url);
	(function (load) {
		var load = outer;
		expect(load("./value")).toBe("dependency");
		load = request => request;
		expect(load("./missing-copy")).toBe("./missing-copy");
	})(outer);
	expect(outer("./value")).toBe("dependency");
});

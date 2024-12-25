import {value} from "./module";

it("should have correct export from re-exports", () => {
	expect(value).toBe("foo");

	const orphanModules = new Set(__STATS__.modules.filter(m => m.orphan).map(m => m.name));
	if (WATCH_STEP === '1') {
		expect(orphanModules).toEqual(new Set(["./module.js", "./reexports.js", "./reexports-deep.js"]));
	} else {
		expect(orphanModules).toEqual(new Set(["./reexports.js", "./reexports-deep.js"]));
	}
});

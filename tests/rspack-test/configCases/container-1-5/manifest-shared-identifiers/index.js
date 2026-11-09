const fs = require("fs");
const path = require("path");

it("should load the shared module", async () => {
	const { value } = await import("./module.js");
	expect(value).toBe("shared");
});

for (const suffix of ["", "-stats"]) {
	it(`should preserve shared fields and assets in mf-${CASE_INDEX}${suffix}.json`, () => {
		const data = JSON.parse(
			fs.readFileSync(path.join(__dirname, `mf-${CASE_INDEX}${suffix}.json`), "utf-8"),
		);
		expect(data.shared).toHaveLength(1);
		const [shared] = data.shared;
		expect(shared).toMatchObject({
			id: `container${CASE_INDEX}:${SHARE_KEY}`,
			name: SHARE_KEY,
			version: SHARED_VERSION,
			requiredVersion: null,
		});
		expect(shared.assets.js.sync.length).toBeGreaterThan(0);
		if (suffix === "-stats" && CONSUME_SHARED) {
			expect(shared.usedIn).toContain("module.js");
		}
	});
}

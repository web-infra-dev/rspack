const p = require("./p");
const m = require("./m");

it("updated modules re-instantiated during dispose keep their parent edges", async () => {
	expect(m.value).toBe(1);

	await NEXT_HMR();

	const updated = require("./m");
	expect(updated.value).toBe(2);
	expect(updated.getModule().parents).toContain("./p.js");
	expect(p.getModule().children).toContain("./m.js");
});

require("./a");
module.hot.accept("./a");

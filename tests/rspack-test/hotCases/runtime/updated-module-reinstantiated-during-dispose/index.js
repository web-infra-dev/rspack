const u = require("./u");

it("updated modules must not keep serving re-instantiated old instances", async () => {
	expect(u.value).toBe(1);

	await NEXT_HMR();

	const updated = require("./u");
	expect(updated).not.toBe(u);
	expect(updated.value).toBe(2);
});

require("./a");
module.hot.accept(["./a", "./u"]);

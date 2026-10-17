const u = require("./u");

it("requiring an updated module while disposing must not serve the old instance", async () => {
	expect(u.value).toBe(1);

	await NEXT_HMR();

	// r was removed by the update and re-required u (which is updated by
	// the same update) from its dispose handler. After the update settles,
	// u must be served from the new factory instead of a stale instance.
	const updated = require("./u");
	expect(updated).not.toBe(u);
	expect(updated.value).toBe(2);
});

require("./a");
module.hot.accept(["./a", "./u"]);

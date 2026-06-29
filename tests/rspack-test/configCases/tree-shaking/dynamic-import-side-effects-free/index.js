const fs = require("fs");
const path = require("path");

function readOutput() {
	return fs
		.readdirSync(path.dirname(__filename))
		.filter((file) => file.endsWith(".js"))
		.map((file) => fs.readFileSync(path.join(path.dirname(__filename), file), "utf-8"))
		.join("\n");
}

function marker(type, mode) {
	return [type, mode, "marker"].join("-");
}

it("should remove unused side-effects-free dynamic import calls", async () => {
	// the unusedAwait variable must keep
	const unusedAwait = await import(/* webpackChunkName: "unused-await" */ "lib/unused-await");
	let thenCalled = false;
	await import(/* webpackChunkName: "unused-then" */ "lib/unused-then").then((m) => { // the m argument must keep
		thenCalled = true;
	});
	expect(thenCalled).toBe(true);

	const awaited = await import(/* webpackChunkName: "used-await" */ "lib/used-await");
	expect(awaited.value).toBe(marker("used", "await"));

	const thenValue = await import(/* webpackChunkName: "used-then" */ "lib/used-then").then(
		(m) => m.value,
	);
	expect(thenValue).toBe(marker("used", "then"));

	const content = readOutput();
	expect(content).not.toContain(marker("unused", "await"));
	expect(content).not.toContain(marker("unused", "then"));
	expect(content).toContain(marker("used", "await"));
	expect(content).toContain(marker("used", "then"));
});

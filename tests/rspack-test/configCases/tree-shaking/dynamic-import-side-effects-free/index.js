const fs = require("fs");
const path = require("path");

function readOutput() {
	return fs
		.readdirSync(path.dirname(__filename))
		.filter((file) => file.endsWith(".js"))
		.map((file) => fs.readFileSync(path.join(path.dirname(__filename), file), "utf-8"))
		.join("\n");
}

function marker(...args) {
	return [...args, "marker"].join("-");
}

function unusedImportReplacement() {
	return ["Promise", "resolve(/* unused import() */ {})"].join(".");
}

it("should remove unused side-effects-free dynamic import calls", async () => {
	const {} = await import(/* webpackChunkName: "unused-empty" */ "lib/unused-empty");
	// the unusedAwait variable must keep
	const unusedAwait = await import(/* webpackChunkName: "unused-await" */ "lib/unused-await");
	// the unusedAwaitEager variable must keep
	const unusedAwaitEager = await import(
		/* webpackMode: "eager" */ "lib/unused-await-eager"
	);
	let thenCalled = false;
	await import(/* webpackChunkName: "unused-then" */ "lib/unused-then").then((m) => { // the m argument must keep
		thenCalled = true;
	});
	expect(thenCalled).toBe(true);
	let thenEagerCalled = false;
	await import(/* webpackChunkName: "unused-then-eager" */ "lib/unused-then-eager").then((m) => { // the m argument must keep
		thenEagerCalled = true;
	});
	expect(thenEagerCalled).toBe(true);

	const awaited = await import(/* webpackChunkName: "used-await" */ "lib/used-await");
	expect(awaited.value).toBe(marker("used", "await"));
	const awaitedEager = await import(/* webpackMode: "eager", webpackChunkName: "used-await-eager" */ "lib/used-await-eager");
	expect(awaitedEager.value).toBe(marker("used", "await", "eager"));

	const thenValue = await import(/* webpackChunkName: "used-then" */ "lib/used-then").then(
		(m) => m.value,
	);
	expect(thenValue).toBe(marker("used", "then"));
	const thenEagerValue = await import(/* webpackMode: "eager", webpackChunkName: "used-then-eager" */ "lib/used-then-eager").then(
		(m) => m.value,
	);
	expect(thenEagerValue).toBe(marker("used", "then", "eager"));

	const content = readOutput();
	expect(content.split(unusedImportReplacement()).length - 1).toBe(5);
	expect(content).not.toContain(marker("unused", "await"));
	expect(content).not.toContain(marker("unused", "await", "eager"));
	expect(content).not.toContain(marker("unused", "empty"));
	expect(content).not.toContain(marker("unused", "then"));
	expect(content).not.toContain(marker("unused", "then", "eager"));
	expect(content).toContain(marker("used", "await"));
	expect(content).toContain(marker("used", "await", "eager"));
	expect(content).toContain(marker("used", "then"));
	expect(content).toContain(marker("used", "then", "eager"));
});

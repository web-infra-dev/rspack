const events = require("./state");
require("./self");
require("./module");

it("should wait for accept promises before reloading self-accepted modules", async () => {
	events.length = 0;
	let release;
	const pending = new Promise(resolve => { release = resolve; });
	let started;
	const accepting = new Promise(resolve => { started = resolve; });
	module.hot.accept("./module", () => {
		events.push("accept");
		started();
		return pending.then(() => { events.push("accepted"); });
	});
	let finished = false;
	const updating = NEXT_HMR().then(() => {
		finished = true;
		events.push("idle");
	});
	await accepting;
	await new Promise(resolve => setTimeout(resolve, 0));
	const beforeRelease = { finished, events: events.slice(), status: module.hot.status() };
	release();
	await updating;
	expect(beforeRelease).toEqual({ finished: false, events: ["accept"], status: "apply" });
	expect(events).toEqual(["accept", "accepted", "self", "idle"]);
});

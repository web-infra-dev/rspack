import "./producer";

const fs = require("fs");

it("drops an unused call to a consumer-side pureFunctions-hinted callee (and its nested pure arg)", () => {
	const source = fs.readFileSync(__filename, "utf-8");

	// Both modules must be tree-shaken: `identity` (pure ONLY via the consumer-side
	// trust-at-call-site Direct path — it is not auto-detectable as pure) and the
	// side-effects-free `payload`. If the Direct path regresses to deferred
	// resolution, `identity` is classified impure and the call — carrying these
	// sentinels — survives, failing the assertions below.
	const wrapperSentinel = ["pure-functions-consumer-hint", "wrapper-sentinel"].join("::");
	const payloadSentinel = ["pure-functions-consumer-hint", "payload-sentinel"].join("::");
	expect(source).not.toContain(wrapperSentinel);
	expect(source).not.toContain(payloadSentinel);
});

import "./changed.js";
import "./stable.js";

it("should retain the stable module id after a content-only edit", () => {
	const module = __STATS__.modules.find(module => module.name === "./stable.js");
	if (WATCH_STEP === "0") STATE.stableId = module.id;
	else expect(module.id).toBe(STATE.stableId);
});

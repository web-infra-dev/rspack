import "./trigger.js";
import "./module7.js";

it("should keep collision ids equal to a full recompute", () => {
	const module = __STATS__.modules.find(module => module.name === "./module7.js");
	expect(module.id).toBe(WATCH_STEP === "0" ? 764 : 383);
});

import { effects } from "./effects";

export * from "./value";

Object.values({
	get a() {
		effects.push("getters");
		return 1;
	}
});

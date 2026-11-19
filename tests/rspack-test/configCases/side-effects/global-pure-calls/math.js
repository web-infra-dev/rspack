import { effects } from "./effects";

export * from "./value";

const Math = {
	log() {
		effects.push("math");
	}
};

Math.log();

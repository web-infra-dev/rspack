import { effects } from "./effects";

export * from "./value";

new Uint8Array({
	get length() {
		effects.push("typed");
		return 0;
	}
});

import { effects } from "./effects";

export * from "./value";

new Map([
	{
		get 0() {
			effects.push("entries");
			return 1;
		},
		1: 2
	}
]);

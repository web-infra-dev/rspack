import { effects } from "./effects";

export * from "./value";

Array.from([1], () => {
	const pushed = effects.push("declared");
	return pushed;
});

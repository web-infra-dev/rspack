import { effects } from "./effects";

export * from "./value";

new Promise(() => {
	effects.push("promise");
});

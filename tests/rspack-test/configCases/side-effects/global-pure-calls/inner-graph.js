import { effects } from "./effects";

export function value() {
	return 42;
}

export const coerced = String({
	toString() {
		effects.push("inner-graph");
		return "";
	}
});

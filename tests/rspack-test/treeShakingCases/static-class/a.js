import { bb, cc } from "./b.js";

export class Test {
	static c = bb();
	static test() {
		bb;
	}
}

export default class Result {
	static test() {
		cc;
	}
}

export const a = 3;

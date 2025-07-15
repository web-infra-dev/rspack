import { A, a, B, b, C, c, D, d, E, e, F, f, X } from "./module";

class Unused {
	[a()]() {
		return A;
	}
	[b] = B;
	get [c]() {
		return C;
	}
	static [d()]() {
		return D;
	}
	static [e] = E;
	static get [f]() {
		return F;
	}
}

class Unused2 extends X {
	[a()]() {
		return A;
	}
	[b] = B;
	get [c]() {
		return C;
	}
	static [d()]() {
		return D;
	}
	static [e] = E;
	static get [f]() {
		return F;
	}
}

export {};

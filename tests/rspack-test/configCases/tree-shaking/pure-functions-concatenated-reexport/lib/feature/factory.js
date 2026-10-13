import "./effect";

export function createWidget(Base) {}

export function impure() {
	globalThis.__concatenatedImpureCalls =
		(globalThis.__concatenatedImpureCalls || 0) + 1;
}

export function testTypeofRequire() {
	if (typeof require !== "undefined") {
		return true;
	}
	return false;
}

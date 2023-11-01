export function assertNotNill(value: any): asserts value {
	if (value == null) {
		throw Error(`${value} should not be undefined or null`);
	}
}

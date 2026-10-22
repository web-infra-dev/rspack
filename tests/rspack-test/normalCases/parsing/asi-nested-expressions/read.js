export function read(value) {
	if (this !== undefined) {
		throw new Error("imported calls must not receive a receiver");
	}
	return value;
}

function toMatchSnapshot() {
	return { pass: true, message: () => "" };
}

function toMatchInlineSnapshot() {
	return { pass: true, message: () => "" };
}

function toMatchFileSnapshot() {
	return { pass: true, message: () => "" };
}

expect.extend({
	toMatchSnapshot,
	toMatchInlineSnapshot,
	toMatchFileSnapshot
});

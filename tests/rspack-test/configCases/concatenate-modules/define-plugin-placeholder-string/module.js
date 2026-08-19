export let value = 42;

if (globalThis.CHANGE_VALUE) {
	value = 43;
}

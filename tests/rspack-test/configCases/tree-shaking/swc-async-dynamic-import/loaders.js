export async function loadDestructured() {
	const { default: value, usedExports } = await import("./module?destructured");
	return { value, usedExports };
}

export async function loadMember() {
	const value = (await import("./module?member")).a;
	const usedExports = (await import("./module?member")).usedExports;
	return { value, usedExports };
}

async function task() {
	await new Promise(res => {
		setTimeout(res, 100);
	});
	return 100;
}

export async function main() {
	await task();
	console.log("hello world!".replaceAll("o", "t"));
}

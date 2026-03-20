export function log(msg) {
	console.log(msg);
}

export function createApp(page) {
	return { page, render: () => log("rendered: " + page) };
}

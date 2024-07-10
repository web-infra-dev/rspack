export function sleep(time: number) {
	return new Promise<void>(res => {
		setTimeout(res, time);
	});
}

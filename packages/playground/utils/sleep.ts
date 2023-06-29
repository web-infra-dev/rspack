export function sleep(time: number) {
	return new Promise<void>(function (res) {
		setTimeout(res, time);
	});
}

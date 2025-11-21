const worker = new Worker(new URL('./my-worker.worker.js', import.meta.url));

let rx;
const promise = new Promise(resolve => {
	rx = resolve;
});

it('should compile success', async () => {
	const res = await promise;

	expect(res).toBe('ok')
})

worker.onmessage = (e) => {
  rx(e.data)
}

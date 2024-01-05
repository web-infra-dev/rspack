it('output js should execute when chunk name has `"` or `\\`', async () => {
	await expect(import(`./a`)).resolves.toMatchObject({ a: 1 });
});

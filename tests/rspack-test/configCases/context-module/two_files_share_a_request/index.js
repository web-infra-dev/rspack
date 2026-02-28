it("should import context module", async () => {
  let n = "index.js"
  const { default: js } = await import(`./sub/${n}`);
  expect(js).toBe("js")

	n = "index.ts"
	const { default: ts } = await import(`./sub/${n}`);
	expect(ts).toBe("ts")
})

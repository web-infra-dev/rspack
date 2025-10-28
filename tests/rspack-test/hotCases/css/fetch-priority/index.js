it("should work", async () => {
	const done = err => (err ? reject(err) : resolve());
	let styles = await import(/* webpackFetchPriority: "high" */ "./style.module.css");

	expect(styles).toMatchObject({
		class: "_style_module_css-class"
	});

	await NEXT_HMR();
	styles = await import(/* webpackFetchPriority: "high" */ "./style.module.css");
	expect(styles).toMatchObject({
		"class-other": "_style_module_css-class-other"
	});

	const links = window.document.getElementsByTagName('link');

	if (links.length > 0) {
		expect(links[0].getAttribute('fetchpriority')).toBe('high');
	}
});

module.hot.accept("./style.module.css");

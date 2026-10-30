import "./shared.css";

it("should reuse initial CSS shared with an async chunk", async () => {
	const initialCssUrl = getInitialCssUrl();
	expect(getCssRequests().filter(url => url === initialCssUrl)).toHaveLength(1);

	const [feature, extra] = await Promise.all([
		import(/* webpackChunkName: "Feature" */ "./feature.js"),
		import(/* webpackChunkName: "Extra" */ "./extra.js")
	]);

	expect(feature.feature).toBe(true);
	expect(extra.extra).toBe(true);
	expect(getCssRequests().some(url => url.includes("undefined"))).toBe(false);
	expect(getCssRequests().filter(url => url === initialCssUrl)).toHaveLength(1);
});

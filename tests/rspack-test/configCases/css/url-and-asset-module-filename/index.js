it(`should generate correct url public path with css filename`, async () => {
	const path = __non_webpack_require__("path");
	const h1 = document.createElement('h1');
	document.body.appendChild(h1);
	const h2 = document.createElement('h2');
	document.body.appendChild(h1);
	const h3 = document.createElement('h3');
	document.body.appendChild(h1);

	let publicPath = '';
	switch (__STATS_I__) {
		case 0:
			publicPath = "../../bundle0/";
			break;
		case 1:
			publicPath = "https://test.cases/path/bundle1/";
			break;
		case 2:
			publicPath = "https://test.cases/path/bundle2/assets/bundle2/";
			break;
	}
	await import("./index.css").then(x => {
		expect(Object.keys(x)).toEqual([]);
		const css = getLinkSheet(document.querySelector("link"));
		expect(css).toContain(`h1 {
  same-dir: url(${publicPath}assets/img1.png);
  nested-dir: url(${publicPath}assets/img2.png);
  nested-nested-dir: url(${publicPath}assets/img3.png);
}`);
		expect(css).toContain(`h2 {
  same-dir: url(${publicPath}assets/img2.png);
  nested-dir: url(${publicPath}assets/img3.png);
  outer-dir: url(${publicPath}assets/img1.png);
}`);
		expect(css).toContain(`h3 {
  same-dir: url(${publicPath}assets/img3.png);
  outer-dir: url(${publicPath}assets/img2.png);
  outer-outer-dir: url(${publicPath}assets/img1.png);
}`);
	});
});

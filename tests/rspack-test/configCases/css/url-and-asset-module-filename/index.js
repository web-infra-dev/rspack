it(`should generate correct url public path with css filename`, async () => {
	const path = require("path");
	const h1 = document.createElement('h1');
	document.body.appendChild(h1);
	const h2 = document.createElement('h2');
	document.body.appendChild(h1);
	const h3 = document.createElement('h3');
	document.body.appendChild(h1);

	let assetPath = '';
	switch (__STATS_I__) {
		case 0:
			assetPath = "../../bundle0/assets/";
			break;
		case 1:
			assetPath = "https://test.cases/path/bundle1/assets/";
			break;
		case 2:
			assetPath = "https://test.cases/path/bundle2/assets/";
			break;
		case 3:
			assetPath = "./img/";
			break;
	}
	await import("./index.css").then(x => {
		expect(Object.keys(x)).toEqual([]);
		const css = getLinkSheet(document.querySelector("link"));
		expect(css).toContain(`h1 {
  same-dir: url("${assetPath}img1.png");
  nested-dir: url("${assetPath}img2.png");
  nested-nested-dir: url("${assetPath}img3.png");
}`);
		expect(css).toContain(`h2 {
  same-dir: url("${assetPath}img2.png");
  nested-dir: url("${assetPath}img3.png");
  outer-dir: url("${assetPath}img1.png");
}`);
		expect(css).toContain(`h3 {
  same-dir: url("${assetPath}img3.png");
  outer-dir: url("${assetPath}img2.png");
  outer-outer-dir: url("${assetPath}img1.png");
}`);
	});
});

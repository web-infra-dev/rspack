import "./style.css";

it(`should work with URLs in CSS`, () => {
	const links = Array.from(document.getElementsByTagName("link"));
	const css = [];

	for (const link of links) {
		css.push(getLinkSheet(link));
	}

	expect(css).toMatchFileSnapshotSync(`${__SNAPSHOT__}/css.${__STATS_I__}.txt`);
});

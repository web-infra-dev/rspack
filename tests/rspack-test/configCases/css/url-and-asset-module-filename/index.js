it(`should generate correct url public path with css filename`, done => {
	const path = __non_webpack_require__("path");
	const h1 = document.createElement('h1');
	document.body.appendChild(h1);
	const h2 = document.createElement('h2');
	document.body.appendChild(h1);
	const h3 = document.createElement('h3');
	document.body.appendChild(h1);
	import("./index.css").then(x => {
		try {
			expect(Object.keys(x)).toEqual([]);
			const style1 = getComputedStyle(h1);
			expect(style1).toMatchFileSnapshot(path.join(__SNAPSHOT__, `style1.${__STATS_I__}.txt`));
			const style2 = getComputedStyle(h2);
			expect(style2).toMatchFileSnapshot(path.join(__SNAPSHOT__, `style2.${__STATS_I__}.txt`));
			const style3 = getComputedStyle(h3);
			expect(style3).toMatchFileSnapshot(path.join(__SNAPSHOT__, `style3.${__STATS_I__}.txt`));
			done();
		} catch (e) {
			done(e);
		}
	}, done);
});

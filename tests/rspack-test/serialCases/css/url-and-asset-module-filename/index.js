it(`should generate correct url public path with css filename`, done => {
	const h1 = document.createElement('h1');
	document.body.appendChild(h1);
	const h2 = document.createElement('h2');
	document.body.appendChild(h1);
	const h3 = document.createElement('h3');
	document.body.appendChild(h1);
	import("./index.css").then(x => {
		try {
			expect(x).toEqual(nsObj({}));
			const style1 = getComputedStyle(h1);
			expect(style1).toMatchFileSnapshot(`${__SNAPSHOT__}/${__STATS_I__}_style1.txt`);
			const style2 = getComputedStyle(h2);
			expect(style2).toMatchFileSnapshot(`${__SNAPSHOT__}/${__STATS_I__}_style2.txt`);
			const style3 = getComputedStyle(h3);
			expect(style3).toMatchFileSnapshot(`${__SNAPSHOT__}/${__STATS_I__}_style3.txt`);
			done();
		} catch (e) {
			done(e);
		}
	}, done);
});

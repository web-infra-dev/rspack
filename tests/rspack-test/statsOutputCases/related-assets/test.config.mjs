export default {
	validate(stats) {
		const { compilation } = stats.stats.find(
			({ compilation }) => compilation.name === "exclude3"
		);
		const { source } = compilation.getAsset("exclude3-main.css.map");
		const map = JSON.parse(source.source().toString());

		expect(map.sources).toEqual([
			expect.stringMatching(/^(webpack|rspack):\/\/\/\.\/style\.css$/)
		]);
	}
};

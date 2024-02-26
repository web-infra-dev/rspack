import { RspackCssExtractPlugin } from "../../../../src";

function recursiveIssuer(m, c) {
	const issuer = c.moduleGraph.getIssuer(m);
	if (issuer) {
		return recursiveIssuer(issuer, c);
	}

	const chunks = c.chunkGraph.getModuleChunks(m);

	for (const chunk of chunks) {
		return chunk.name;
	}

	return false;
}

module.exports = {
	entry: {
		a: "./a.js",
		b: "./b.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [RspackCssExtractPlugin.loader, "css-loader"]
			}
		]
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				aStyles: {
					name: "styles_a",
					test: (m, c, entry = "a") => {
						return (
							typeof m.identifier === "string" &&
							m.identifier.endsWith(".css") &&
							recursiveIssuer(m, c) === entry
						);
					},
					chunks: "all",
					enforce: true
				},
				bStyles: {
					name: "styles_b",
					test: (m, c, entry = "b") =>
						typeof m.identifier === "string" &&
						m.identifier.endsWith(".css") &&
						recursiveIssuer(m, c) === entry,
					chunks: "all",
					enforce: true
				}
			}
		}
	},
	plugins: [new RspackCssExtractPlugin()]
};

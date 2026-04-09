const path = require("path");

module.exports = function (content, sourceMap, additionalData) {
	const resource = path.basename(this.resourcePath);

	expect(sourceMap).toBeTruthy();

	if (resource === "swc.jsx") {
		expect(content).toContain("React.createElement");
	} else if (resource === "react-refresh.jsx") {
		expect(content).toContain(
			"$ReactRefreshRuntime$.createSignatureFunctionForTransform()"
		);
	} else if (resource === "preact-refresh.jsx") {
		expect(content).toContain("__prefresh_utils__");
	} else if (resource === "lightning.css") {
		expect(content.replace(/\s+/g, "")).toContain("body{color:red}");
	}

	this.callback(
		null,
		`module.exports = ${JSON.stringify({
			...additionalData,
			fromLoader2: true
		})}`,
		null
	);
};

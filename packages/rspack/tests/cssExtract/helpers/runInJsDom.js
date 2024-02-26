import jsdom from "jsdom";

import { readAsset } from "./index";

function runInJsDom(assetName, compiler, stats, testFn) {
	const bundle = readAsset(assetName, compiler, stats);
	const virtualConsole = new jsdom.VirtualConsole();

	virtualConsole.sendTo(console);

	try {
		const dom = new jsdom.JSDOM(
			`<!doctype html>
<html>
<head>
  <title>style-loader test</title>
  <style id="existing-style">.existing { color: yellow }</style>
</head>
<body>
  <h1>Body</h1>
  <div class="target"></div>
  <iframe class='iframeTarget'></iframe>
</body>
</html>
`,
			{
				resources: "usable",
				runScripts: "dangerously",
				virtualConsole
			}
		);

		dom.window.eval(bundle);

		testFn(dom, bundle);

		// free memory associated with the window
		dom.window.close();
	} catch (e) {
		throw e;
	}
}

export default runInJsDom;

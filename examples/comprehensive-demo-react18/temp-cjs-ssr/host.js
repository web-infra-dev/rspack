const React = require("react");
const ReactDOMServer = require("react-dom/server");

async function run() {
	const remote = await import("remote/Widget");
	const Widget = remote.default || remote;
	const element = React.createElement(
		"div",
		null,
		React.createElement("h2", null, "SSR host rendering"),
		React.createElement(Widget, { who: "server" })
	);
	const html = ReactDOMServer.renderToString(element);
	console.log("[SSR HTML]\n" + html);
}

run().catch(err => {
	console.error("SSR failed", err);
	process.exit(1);
});

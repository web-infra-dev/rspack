import React from "react";
import ReactDOM from "react-dom/client";

import(/* webpackChunkName: "AppIndex" */"./AppIndex").then(({ default: App }) => {
	global.webpackRequire = __webpack_modules__;
	ReactDOM.createRoot(document.getElementById("root")).render(
		<React.StrictMode>
			<App />
		</React.StrictMode>
	);
});

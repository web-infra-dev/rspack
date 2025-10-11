import React from "react";
import ReactDOM from "react-dom/client";

ReactDOM.createRoot(document.getElementById("root")).render(
	<React.StrictMode>
		<button type="button" onClick={() => {
			import("./remote-entry.js");
			import("./share-entry.js");
		}}>
			Click me
		</button>
	</React.StrictMode>
);


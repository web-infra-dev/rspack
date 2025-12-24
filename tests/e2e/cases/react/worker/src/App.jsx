import React, { useState } from "react";
import Button from "./Button";

let updateRenderTimes;

const worker = new Worker(new URL("./worker", import.meta.url));

worker.onmessage = e => {
	updateRenderTimes(e.data);
};

export const App = () => {
	const [renderTimes, setRenderTimes] = useState(0);
	updateRenderTimes = setRenderTimes;
	return (
		<div className="App">
			<h1>{renderTimes}</h1>
			<Button onClick={() => worker.postMessage("add")} />
		</div>
	);
};

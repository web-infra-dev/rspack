import react from "react";
import remote from "remote";

global.react = react;
global.remote = remote;

import("./lazy-module").then(mod => {
	console.log("lazy module", mod);
});

export const ok = true;

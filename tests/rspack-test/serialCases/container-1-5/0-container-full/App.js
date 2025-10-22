import React from "mocked-react";
import ComponentA from "containerA/ComponentA";

console.log("debug: React is", React)

export default () => {
	return `App rendered with [${React()}] and [${ComponentA()}]`;
};

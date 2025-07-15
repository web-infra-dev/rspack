import ComponentA from "containerA/ComponentA";
import ComponentB from "containerB/ComponentB";
import React from "react";

export default () => {
	return `ComponentC rendered with [${React()}] and [${ComponentA()}] and [${ComponentB()}]`;
};

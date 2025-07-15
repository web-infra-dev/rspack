import ComponentA from "containerA/ComponentA";
import ComponentB from "containerB/ComponentB";
import React from "react";
import LocalComponentB from "./ComponentB";

export default () => {
	return `App rendered with [${React()}] and [${ComponentA()}] and [${ComponentB()}]`;
};

expect(ComponentB).not.toBe(LocalComponentB);

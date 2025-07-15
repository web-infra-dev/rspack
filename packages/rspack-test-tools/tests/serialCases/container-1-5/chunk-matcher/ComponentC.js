import React from "react";
import ComponentA from "containerA/ComponentA";

export default () => {
	return `ComponentC rendered with [${React()}] and [${ComponentA()}]`;
};

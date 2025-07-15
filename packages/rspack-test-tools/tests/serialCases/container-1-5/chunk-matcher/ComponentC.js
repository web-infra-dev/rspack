import ComponentA from "containerA/ComponentA";
import React from "react";

export default () => {
	return `ComponentC rendered with [${React()}] and [${ComponentA()}]`;
};

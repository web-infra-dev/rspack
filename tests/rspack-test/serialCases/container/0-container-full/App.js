import React from "react";
import ComponentA from "containerA/ComponentA";
console.log('react:',React);
export default () => {
	return `App rendered with [${React()}] and [${ComponentA()}]`;
};

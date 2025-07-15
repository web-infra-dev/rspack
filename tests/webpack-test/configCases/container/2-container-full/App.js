import ComponentC from "containerB/ComponentC";
import OldReact from "old-react";
import OldReactSingleton from "old-react-singleton";
import React from "react";

export default () => {
	return `App rendered with [${React()}] and [${OldReact()}] and [${OldReactSingleton()}] and [${ComponentC()}]`;
};

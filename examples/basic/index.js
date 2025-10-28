import "./lib";
import React from "react";

console.log("React version:", React.version);

// Test React hooks
const [count, _setCount] = React.useState(0);
console.log("useState initialized with:", count);

// Test createElement
const element = React.createElement(
	"div",
	{ className: "test" },
	"Hello from Module Federation!"
);
console.log("Created element:", element);

// Component module that uses React
import React from "react";

export function HelloComponent(props) {
	return React.createElement(
		"div",
		{ className: "hello" },
		`Hello, ${props.name || "World"}!`
	);
}

export function ListComponent(props) {
	return React.createElement(
		"ul",
		{ className: "list" },
		props.items.map((item, index) =>
			React.createElement("li", { key: index }, item)
		)
	);
}

// Both components use React.createElement
// This should show React as used in ShareUsagePlugin output

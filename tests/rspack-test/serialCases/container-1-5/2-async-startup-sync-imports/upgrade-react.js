import React, { setVersion } from "react";

export default function upgrade() {
	// Detect current version and upgrade accordingly
	const current = React();
	if (current.includes("2.1.0")) {
		setVersion("3.2.1");
	} else if (current.includes("3.2.1")) {
		setVersion("4.3.2");
	}
}

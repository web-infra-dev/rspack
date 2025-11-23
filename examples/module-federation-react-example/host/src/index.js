// Module Federation async boundary - dynamic import to bootstrap
import("./bootstrap.jsx").catch(err =>
	console.error("Error loading bootstrap:", err)
);

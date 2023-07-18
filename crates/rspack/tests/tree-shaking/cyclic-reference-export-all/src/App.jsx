import React from "react";

import { containers } from "./containers";
const { PlatformProvider } = containers;

const Index = () => {
	console.log("PlatformProvider", PlatformProvider);
	return <div>something</div>;
};

export default Index;

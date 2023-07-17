import React from "react";

// ----有bug----
import { containers } from "./containers";
// import * as domains from '@ecom/cross-border-domains';
const { PlatformProvider } = containers;

const Index = () => {
	console.log("PlatformProvider", PlatformProvider);
	return <div>something</div>;
};

export default Index;

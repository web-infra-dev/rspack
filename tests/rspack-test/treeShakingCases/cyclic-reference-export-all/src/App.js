import { containers } from "./containers";
const { PlatformProvider } = containers;

const Index = () => {
	console.log("PlatformProvider", PlatformProvider);
	return "something";
};

export default Index;

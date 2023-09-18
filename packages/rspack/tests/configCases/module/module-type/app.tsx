const React = {
	createElement(elm: any) {}
};
function Component<T>() {
	return <div></div>;
}

export function App() {
	return <Component<any>></Component>;
}

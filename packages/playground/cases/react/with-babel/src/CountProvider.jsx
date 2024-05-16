import React from "react";

export const CountContext = React.createContext();

export function CountProvider({ children }) {
	const [count, setCount] = React.useState("context-value");
	return (
		<CountContext.Provider value={{ count, setCount }}>
			{children}
		</CountContext.Provider>
	);
}

export function ContextComponent() {
	const { count, setCount } = React.useContext(CountContext);
	return (
		<div id="context" onClick={() => setCount(count => count + "-click")}>
			{count}
		</div>
	);
}

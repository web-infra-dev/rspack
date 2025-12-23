import React from "react";

const Button = () => {
	const [count, setCount] = React.useState(10);
	return (
		<button type="button" onClick={() => setCount(count => count + 1)}>
			{count}
		</button>
	);
};

export class App extends React.Component {
	render() {
		return (
			<div className="App">
				<div className="header">Hello World</div>
				<Button />
				<div className="placeholder">__PLACE_HOLDER__</div>
			</div>
		);
	}
}

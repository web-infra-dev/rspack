import React from "react";

export default function Button({ onClick }) {
	return <button onClick={onClick}>+</button>;
}

Button.count = 0;

Button.get = () => {
	return Button.count;
};

Button.add = () => {
	Button.count += 1;
};

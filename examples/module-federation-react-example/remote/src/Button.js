import { capitalize } from "lodash-es";

export default function Button({ text, onClick }) {
	return (
		<button
			type="button"
			onClick={onClick}
			style={{
				padding: "10px 20px",
				backgroundColor: "#007acc",
				color: "white",
				border: "none",
				borderRadius: "4px",
				cursor: "pointer",
				fontSize: "16px"
			}}
		>
			{capitalize(text)}
		</button>
	);
}

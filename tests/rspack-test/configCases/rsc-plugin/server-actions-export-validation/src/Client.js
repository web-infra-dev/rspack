"use client";

import { invalidAction } from "./invalid-actions";

export const Client = () => {
	async function onClick() {
		await invalidAction();
	}

	return (
		<button type="button" onClick={onClick}>
			Run actions
		</button>
	);
};

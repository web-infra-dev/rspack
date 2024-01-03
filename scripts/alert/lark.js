#!/bin/env node

const TITLE = process.env.TITLE;
const DESCRIPTION = process.env.DESCRIPTION;
const URL = process.env.URL;
const LARK_WEBHOOK_URL = process.env.LARK_WEBHOOK_URL;

if (!TITLE || !DESCRIPTION) {
	throw new Error("please input title and description");
}

const res = await fetch(LARK_WEBHOOK_URL, {
	method: "POST",
	headers: {
		"Content-Type": "application/json"
	},
	body: JSON.stringify({
		msg_type: "interactive",
		card: {
			header: {
				template: "red",
				title: {
					content: TITLE,
					tag: "plain_text"
				}
			},
			elements: [
				{
					tag: "markdown",
					content: DESCRIPTION
				},
				URL && {
					tag: "action",
					actions: [
						{
							tag: "button",
							text: {
								content: "Details",
								tag: "plain_text"
							},
							url: URL,
							type: "danger"
						}
					]
				}
			].filter(Boolean)
		}
	})
});

if (!res.ok) {
	const data = await res.text();
	throw new Error("send alert failed with " + data);
}

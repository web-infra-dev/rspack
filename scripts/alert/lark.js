#!/bin/env node

const TITLE = process.env.TITLE;
const DESCRIPTION = process.env.DESCRIPTION;
const URL = process.env.URL;
const LARK_WEBHOOK_URL = process.env.LARK_WEBHOOK_URL;
const TPL_COLOR = process.env.TPL_COLOR || "red";
const TPL_BTN_TYPE = process.env.TPL_BTN_TYPE || "danger"; // default primary danger

if (!TITLE || !DESCRIPTION) {
	throw new Error("please input title and description");
}

if (!LARK_WEBHOOK_URL) {
	console.log("missing LARK_WEBHOOK_URL, will exit");
	process.exit(0);
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
				template: TPL_COLOR,
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
							type: TPL_BTN_TYPE
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

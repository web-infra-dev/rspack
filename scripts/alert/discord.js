#!/bin/env node

const TITLE = process.env.TITLE;
const DESCRIPTION = process.env.DESCRIPTION;
const URL = process.env.URL;
const DISCORD_WEBHOOK_URL = process.env.DISCORD_WEBHOOK_URL;

if (!TITLE || !DESCRIPTION) {
	throw new Error("please input title and description");
}

const res = await fetch(DISCORD_WEBHOOK_URL, {
	method: "POST",
	headers: {
		"Content-Type": "application/json"
	},
	body: JSON.stringify({
		username: "rspack-bot",
		avatar_url:
			"https://raw.githubusercontent.com/web-infra-dev/rspack/main/website/docs/public/logo.png",
		embeds: [
			{
				title: TITLE,
				description: DESCRIPTION + URL ? `\n* url: ${URL}` : ""
			}
		]
	})
});

if (!res.ok) {
	const data = await res.text();
	throw new Error("send alert failed with " + data);
}

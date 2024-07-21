import ansiHTML from "../src/ansiHTML";

describe("ansi-html", () => {
	it("should transform 24-bit rgb ansi colors", () => {
		expect(ansiHTML("\u001b[38;2;255;30;30m×\u001b[0m")).toMatchInlineSnapshot(
			`"<span style="color: rgb(255,30,30);">×</span><span style="font-weight:normal;opacity:1;color:#fff;background:#000;"></span>"`
		);
	});

	it("should transform 24-bit rgb ansi colors with additional properties", () => {
		expect(
			ansiHTML("[\u001b[38;2;92;157;255;1;4m/root/index.js\u001b[0m:1:1]")
		).toMatchInlineSnapshot(
			`"[<span style="color: rgb(92,157,255);"><span style="font-weight:bold;"><u>/root/index.js</u></span></span><span style="font-weight:normal;opacity:1;color:#fff;background:#000;">:1:1]</span>"`
		);
	});

	it("should transform basic ansi colors", () => {
		expect(ansiHTML("\u001b[0mcontent")).toMatchInlineSnapshot(
			`"<span style="font-weight:normal;opacity:1;color:#fff;background:#000;">content</span>"`
		);
		expect(
			ansiHTML(
				`\u001b[33m<\u001b[39m\u001b[33mheader\u001b[39m\u001b[33m>\u001b[39m`
			)
		).toMatchInlineSnapshot(
			`"<span style="color:#e8bf03;"><</span><span style="color:#e8bf03;">header</span><span style="color:#e8bf03;">></span>"`
		);
	});
});

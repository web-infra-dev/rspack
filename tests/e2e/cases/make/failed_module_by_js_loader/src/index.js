Promise.resolve().then(() => {
	const div = document.createElement("div");
	div.innerText = "index";
	div.id = "index";
	document.body.appendChild(div);
});

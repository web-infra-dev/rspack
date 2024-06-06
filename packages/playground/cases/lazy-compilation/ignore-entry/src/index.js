const button = document.createElement("button");
button.textContent = "Click me";
button.onclick = () => {
	history.pushState(null, "", "success");
};
document.body.appendChild(button);

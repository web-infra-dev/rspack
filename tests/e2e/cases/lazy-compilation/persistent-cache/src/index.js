const button = document.createElement("button");
button.textContent = "Click me";
document.body.appendChild(button);

button.addEventListener("click", async () => {
	import('./dyn')
})

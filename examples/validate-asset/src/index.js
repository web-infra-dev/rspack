const MY_SUPER_SECRET = "Don't share me!";

function render() {
	document.getElementById("root").innerHTML = `Hello "${MY_SUPER_SECRET}"!`;
}
render();

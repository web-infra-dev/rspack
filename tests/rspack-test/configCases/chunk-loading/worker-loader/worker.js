onmessage = async function() {
	let fetch = await import("./worker-async")
	postMessage(`Hello ${fetch.message}`)
}

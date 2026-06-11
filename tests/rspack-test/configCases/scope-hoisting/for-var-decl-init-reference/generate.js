const lightColorCount = 5;

export default function generate() {
	let total = 0;
	for (let i = lightColorCount; i > 0; i -= 1) {
		total += i;
	}
	return total;
}

let i = 0;
export default function(content) {
  this.cacheable(false);
  i++;
  if (i === 1) {
    return content;
  } else {
    return `
		.bar {
			color: red;
		}
    `;
  }
}

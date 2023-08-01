const png = require('./icon.png');
function render() {
    document.getElementById('root').innerHTML = `The answer to the universe is ${answer}.`;
    const img = document.createElement('img');
    img.src = png;
}
render()
import { answer } from './answer';
import {a} from '../a'
function render() {
	document.getElementById(
		"root"
	).innerHTML = `the answer to the universe is ${answer}`;
}
render();
a;

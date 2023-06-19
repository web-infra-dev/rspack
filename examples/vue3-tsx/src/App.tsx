import { FunctionalComponent, ref } from "vue";
import vueLogo from "./assets/vue.svg";
import "./App.css";

export default {
	name: "App",

	setup() {
		const count = ref(0);
		const add = () => count.value++;

		const CounterButton: FunctionalComponent<{
			onClick: () => void;
			value: number;
		}> = ({ onClick, value }) => {
			return <button onClick={onClick}>count is {value}</button>;
		};

		return () => (
			<div class="App">
				<h1>Hello world!</h1>
				<div>
					<a href="https://vuejs.org" target="_blank">
						<img src={vueLogo} class="logo vue" alt="Vue logo" />
					</a>
				</div>
				<h1>Rspack + Vue JSX</h1>
				<div class="card">
					<CounterButton onClick={add} value={count.value} />
				</div>
			</div>
		);
	}
};

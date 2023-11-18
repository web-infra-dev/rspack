<script setup lang="ts">
import { ref } from "vue";
import TSX_Counter from "./components/TSX_Counter";
import SFC_TSX_Counter from "./components/SFC_TSX_Counter.vue";
import SFC_JSX_Counter from "./components/SFC_JSX_Counter.vue";

const title = ref<string>("Vue2 + TSX + Rspack");
</script>

<template>
	<div>
		<h1>{{ title }}</h1>
		<div class="box">
			<TSX_Counter />
			<SFC_TSX_Counter />
			<SFC_JSX_Counter />
		</div>
	</div>
</template>

<style scoped lang="less">
h1 {
	background-size: 400% 400%;
	background-image: linear-gradient(
		to right,
		#ff0000,
		#fff200,
		#00d0ff,
		#9000ff,
		#ff008c
	);
	background-clip: text;
	-webkit-background-clip: text;
	animation: text-color-gradient 10s linear infinite;
	color: transparent;
	margin: -5rem 0 5rem;

	@keyframes text-color-gradient {
		0% {
			background-position: 0% 50%;
		}

		50% {
			background-position: 100% 50%;
		}

		100% {
			background-position: 0% 50%;
		}
	}
}

.box {
	width: min(200px, 80vw);
	margin: 0 auto;
	display: flex;
	flex-direction: column;
	align-items: stretch;
	justify-content: center;
	gap: 1rem;
}
</style>

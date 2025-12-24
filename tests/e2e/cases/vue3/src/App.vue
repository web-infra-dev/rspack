<template>
	<h1>vue3</h1>
	<button id="count" @click="count++">{{ count }}</button>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { Type } from "./enums";

defineProps<{
	__: Type;
}>();
const count = ref(Type.START);
</script>

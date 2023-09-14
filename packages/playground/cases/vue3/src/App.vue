<template>
	<h1>vue3</h1>
	<button id="count" @click="count++">{{ count }}</button>
</template>

<script>
import { ref } from 'vue';
export default {
  setup() {
    const count = ref(0)
    return { count }
  }
}
</script>

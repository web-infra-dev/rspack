<script>
import HelloWorld from './components/HelloWorld.vue';
export default {
  components: { HelloWorld }
}
</script>

<template>
  <div>
    <a href="https://www.rspack.dev/" target="_blank">
      <img src="./assets/rspack.svg" class="logo" alt="Rspack logo" />
    </a>
    <a href="https://vuejs.org/" target="_blank">
      <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
    </a>
  </div>
  <HelloWorld msg="Rspack + Vue" />
</template>

<style scoped lang="less">
.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;

  &:hover {
    filter: drop-shadow(0 0 2em #eeea8caa);
  }

  &.vue:hover {
    filter: drop-shadow(0 0 2em #42b883aa);
  }
}
</style>

<script setup lang="ts">
import init, { greet } from 'emulator'

init()
</script>

<template>
  <header>
    Intel 8080 - Emulator
  </header>

  <main>
    <br/>
    <button class="bg-red-300 m-4 p-4 rounded text-lg" @click="greet()">
      Check
    </button>

    
  </main>
</template>

<style scoped>
header {
  line-height: 1.5;
}

.logo {
  display: block;
  margin: 0 auto 2rem;
}

@media (min-width: 1024px) {
  header {
    display: flex;
    place-items: center;
    padding-right: calc(var(--section-gap) / 2);
  }

  .logo {
    margin: 0 2rem 0 0;
  }

  header .wrapper {
    display: flex;
    place-items: flex-start;
    flex-wrap: wrap;
  }
}
</style>

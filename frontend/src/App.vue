<script setup lang="ts">
import { ref } from 'vue'
import init, { greet, set_disassemble, get_disassemble } from 'emulator'

const disassembleState = ref(false)

function greet_wasm() {
  console.log("Greeting WASM")
  greet()
}

function setDisassemble(flag: boolean) {
  set_disassemble(flag)
  disassembleState.value = get_disassemble()
}

init()
</script>

<template>
  <header>
    Intel 8080 - Emulator
  </header>

  <main>
    <br/>
    <button class="bg-red-300 m-4 p-4 rounded text-lg" @click="greet_wasm()">
      Check
    </button>
    <br/>
    Disassemble State: {{ disassembleState }} <br />
    <button class="bg-red-300 m-4 p-4 rounded text-lg" @click="setDisassemble(true)">
      Disassemble ON
    </button>
    <button class="bg-red-300 m-4 p-4 rounded text-lg" @click="setDisassemble(false)">
      Disassemble OFF
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

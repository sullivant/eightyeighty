<script setup lang="ts">
import { ref } from 'vue'
import init, { cpu_greet, cpu_set_disassemble, cpu_get_disassemble, cpu_memory_write, cpu_state } from 'emulator'

const disassembleState = ref(false)
const cpuState = ref("CPU NOT READY");

function updateInterface() {
  disassembleState.value = cpu_get_disassemble();
  cpuState.value = cpu_state();
}

function greetWASM() {
  console.log("Greeting WASM")
  cpu_greet()
}

function setDisassemble(flag: boolean) {
  cpu_set_disassemble(flag)
  updateInterface();
}

function loadROM() {
  console.log("Loading INVADERS.COM")

  fetch('roms/INVADERS.COM')
    .then(i => i.arrayBuffer())
    .then(buffer => {
      // Loop and write rom to CPU memory
      const start_index = 0;
      const rom = new DataView(buffer, 0, buffer.byteLength);
      for (let i = 0; i < rom.byteLength; i++) {
        cpu_memory_write(start_index+i, rom.getUint8(i));
      }
    });
    updateInterface();
}


init()
</script>

<template>
  <header>
    Intel 8080 - Emulator
  </header>

  <main>
    <br/>
    <button class="bg-red-300 m-4 p-4 rounded text-lg" @click="greetWASM()">
      Check
    </button>
    <br/>
    
    <button class="bg-red-300 m-4 p-4 rounded text-lg" @click="setDisassemble(true)">
      Disassemble ON
    </button>
    <button class="bg-red-300 m-4 p-4 rounded text-lg" @click="setDisassemble(false)">
      Disassemble OFF
    </button>
    <button class="bg-red-300 m-4 p-4 rounded text-lg" @click="loadROM()">
      Load ROM
    </button>

    <footer>
      Disassemble State: {{ disassembleState }} <br />
      CPU State: {{  cpuState }} <br />
    </footer>

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

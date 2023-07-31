<script setup lang="ts">
import { ref } from 'vue'

import init, { cpu_greet, cpu_set_disassemble, cpu_get_disassemble, cpu_memory_write, 
  cpu_get_memory, cpu_state, cpu_curr_instr, cpu_tick } from 'emulator'

const disassembleState = ref(false)
const cpuState = ref("CPU NOT READY");
const currInstr = ref("NO INSTRUCTION");
const currRAM = ref("NO RAM");

function updateInterface() {
  disassembleState.value = cpu_get_disassemble();
  cpuState.value = cpu_state();
  currInstr.value = cpu_curr_instr();
  currRAM.value = cpu_get_memory();
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
    })
    updateInterface()
}

function tick() {
  cpu_tick()
  updateInterface()
}

function getCPUState() {
  cpuState.value = cpu_state();
}


init()

</script>

<template>
  <v-layout class="rounded rounded-md">
    <v-app-bar color="surface-variant" title="8080"></v-app-bar>

    <v-navigation-drawer expand-on-hover rail>
      <v-divider></v-divider>

        <v-list density="compact" nav>
          <v-list-item prepend-icon="mdi-play-circle-outline" title="Load ROM" @click="loadROM"></v-list-item>
          <v-list-item prepend-icon="mdi-check-circle-outline" title="Set Disassemble" @click="setDisassemble(true)"></v-list-item>
          <v-list-item prepend-icon="mdi-alpha-x-circle-outline" title="Unset Disassemble" @click="setDisassemble(false)"></v-list-item>
          <v-list-item prepend-icon="mdi-bug" title="Tick" @click="tick()"></v-list-item>
        </v-list>
    </v-navigation-drawer>

    <v-navigation-drawer location="right">
      <v-list>
        <v-list-item title="Disassemble">{{ disassembleState }}</v-list-item>
      </v-list>
    </v-navigation-drawer>

    <v-main class="d-flex align-center justify-center" style="min-height: 300px;">
      {{ cpuState }}
    </v-main>

    <v-bottom-navigation>
      {{  currInstr }}
    </v-bottom-navigation>
  </v-layout>



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

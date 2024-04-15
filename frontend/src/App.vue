<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import Memory from './components/Memory.vue'

import init, { cpu_greet, cpu_set_disassemble, cpu_get_disassemble, cpu_memory_write, 
  cpu_get_memory, cpu_state, cpu_curr_instr, cpu_tick } from 'emulator'

var tab = null;
const disassembleState = ref(false);
const cpuState = ref("CPU NOT READY");
const currInstr = ref("NO INSTRUCTION");

let currRAM = ref(JSON.parse('[{"address": -1}]'));

function greetWASM() {
  console.log("Greeting WASM")
  cpu_greet()
}

// Toggles the dissassably state (which will update the latest instruction, etc)
// and in doing so calls down to the wasm to actually implement the change.
// (this is not just a local boolean flip.)
function toggleDisassemble() {
  // cpu_set_disassemble(!disassembleState.value)
  disassembleState.value = !disassembleState.value
  cpu_set_disassemble(disassembleState.value)
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
}

function tick() {
  cpu_tick()
}

async function refreshRAMState() {
  console.log("Refreshing CPU memory...");
  currRAM.value=[];

  //Walk through each slice, incrementing address by 0x0F each time and insert it into currRAM.value[]
  for(let i = 0; i < 4096; i++){
    var thisAddress = (i*16);
    const ramState = JSON.parse(cpu_get_memory(thisAddress));   // i*16
    var currSlice = {};
    currSlice["address"] = "0x"+thisAddress.toString(16).toUpperCase().padStart(4,'0');
    ramState.forEach((element, idx) => {
      currSlice["0x"+idx.toString(16).toUpperCase()] = element.toString(16).toUpperCase().padStart(1,'0');
    });
    currRAM.value.push(currSlice);
  }

}

init()

</script>

<template>
  <v-layout class="rounded rounded-md">
    <v-app-bar color="surface-variant" title="8080"></v-app-bar>

    <v-navigation-drawer expand-on-hover rail>
      <v-divider></v-divider>

        <v-list density="compact" nav>
          <v-list-item
            prepend-icon="mdi-robot-love-outline"
            title="Controls"
          />
          <v-divider/>
          <v-list-item prepend-icon="mdi-play-circle-outline" title="Load ROM" @click="loadROM"></v-list-item>
          <v-list-item v-if="disassembleState" prepend-icon="mdi-package-variant" title="Disassembling" @click="toggleDisassemble()"></v-list-item>
          <v-list-item v-else prepend-icon="mdi-package-variant-closed" title="Not Disassembling" @click="toggleDisassemble()"></v-list-item>
          <v-list-item prepend-icon="mdi-bug" title="Tick" @click="tick()"></v-list-item>
          <v-list-item prepend-icon="mdi-memory" title="Refresh RAM" @click="refreshRAMState()"></v-list-item>
        </v-list>
    </v-navigation-drawer>

    <v-navigation-drawer location="right"> 
      <v-list density="compact" nav>
          <v-list-item
            prepend-icon="mdi-puzzle-outline"
            title="Details"
          />
          <v-divider/>
          {{  currInstr }}
          <v-divider/>
          <v-divider/>
          {{ cpuState }}
          <v-divider/>
          {{ tab }}
        </v-list>
    </v-navigation-drawer>

    <v-main class="d-flex align-center justify-center" style="min-height: 300px;">
      <Memory :currRAM=currRAM />
    </v-main>

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

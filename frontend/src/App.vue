<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import Memory from './components/Memory.vue'
import Registers from './components/Registers.vue'

import init, { cpu_greet, cpu_set_disassemble, cpu_get_disassemble, cpu_memory_write, 
  cpu_get_memory, cpu_state, cpu_curr_instr, cpu_tick, get_all_registers, cpu_reset } from 'emulator'

var tab = null;
const disassembleState = ref(false);
const cpuState = ref("CPU NOT READY");
const currInstr = ref("NO INSTRUCTION");

let currRAM = ref([{address: -1}]);

// let currRegisters = ref(JSON.parse('[{"register":"PC", "value":-1}]'))
let currRegisters = ref([
  {register:"PC", value:-1},
  {register:"SP", value:123},
]);

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
  console.log("Loaded INVADERS.COM");

  refreshRAMState();
  refreshRegisters();
}

function reset() {
  console.log("Resetting CPU");
  cpu_reset();
  refreshRAMState();
  refreshRegisters();
  refreshCurrentInstr();
}

function tick() {
  cpu_tick();
  refreshRAMState();
  refreshRegisters();
  refreshCurrentInstr();
}

function makeHex(value, hexlen) {
  return "0x"+value.toString(16).toUpperCase().padStart(hexlen,'0');
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

async function refreshRegisters() {
  // (&self.pc, &self.sp, &self.a, &self.b, &self.c, &self.d, &self.e, &self.h, &self.l)
  console.log("Refreshing Registers...");
  currRegisters.value=[];
  const regState = JSON.parse(get_all_registers());

  currRegisters.value.push({register:"PC", value:makeHex(regState[0], 4)});
  currRegisters.value.push({register:"SP", value:makeHex(regState[1], 4)});
  currRegisters.value.push({register:"A", value:makeHex(regState[2], 4)});
  currRegisters.value.push({register:"B", value:makeHex(regState[3], 4)});
  currRegisters.value.push({register:"C", value:makeHex(regState[4], 4)});
  currRegisters.value.push({register:"D", value:makeHex(regState[5], 4)});
  currRegisters.value.push({register:"E", value:makeHex(regState[6], 4)});
  currRegisters.value.push({register:"H", value:makeHex(regState[7], 4)});
  currRegisters.value.push({register:"L", value:makeHex(regState[8], 4)});
}

async function refreshCurrentInstr() {
  console.log("Refreshing current instruction...");
  currInstr.value=cpu_curr_instr();
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
          <v-list-item prepend-icon="mdi-restart" title="Rest CPU" @click="reset()"></v-list-item>          
          <v-list-item v-if="disassembleState" prepend-icon="mdi-package-variant" title="Disassembling" @click="toggleDisassemble()"></v-list-item>
          <v-list-item v-else prepend-icon="mdi-package-variant-closed" title="Not Disassembling" @click="toggleDisassemble()"></v-list-item>
          <v-list-item prepend-icon="mdi-bug" title="Tick" @click="tick()"></v-list-item>
          <v-list-item prepend-icon="mdi-memory" title="Refresh RAM" @click="refreshRAMState()"></v-list-item>
          <v-list-item prepend-icon="mdi-ab-testing" title="Refresh Registers" @click="refreshRegisters()"></v-list-item>
        </v-list>
    </v-navigation-drawer>

    <v-navigation-drawer location="right"> 
        <Registers :currRegisters=currRegisters :cpuState=cpuState :currInstr=currInstr />
    </v-navigation-drawer>

    <v-main class="d-flex" style="min-height: 300px;">
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

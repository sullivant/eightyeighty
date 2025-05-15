<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useMouse } from '@vueuse/core'
import { useTheme } from 'vuetify'
import Memory from './components/Memory.vue'
import Registers from './components/Registers.vue'
import Display from './components/Display.vue'

import init, { Emulator } from 'emulator'

let emulator: Emulator;

const theme = useTheme();
const drawer = ref(false);
const shouldAutoTick = ref(false);

// let currRAM = ref([{address: -1}]);
const currRAM = ref<Array<[string, Array<[string, number]>]>>([]);

let currRegisters = ref([
  {register:"PC", value:makeHex(-1, 4)},
  {register:"SP", value:makeHex(123, 4)},
]);

let instructions = ref([
  {type:"C", opcode:makeHex(0,2), size:"0", cycles:"0", text: ""},
  {type:"N", opcode:makeHex(0,2), size:"0", cycles:"0", text: ""},
]);

function sleep(ms: number) {
  return new Promise (res => setTimeout(res, ms));
}

function loadROM() {
  reset();
  fetch('roms/INVADERS.COM')
    .then(i => i.arrayBuffer())
    .then(buffer => {
      // Loop and write rom to CPU memory
      const start_index = 0;
      const rom = new DataView(buffer, 0, buffer.byteLength);
      for (let i = 0; i < rom.byteLength; i++) {
        emulator.cpu_memory_write(start_index+i, rom.getUint8(i));
      }
    });
  console.log("Loaded INVADERS.COM");
  sleep(500).then(() => { refreshRAMState(); refreshRegisters(); refreshInstructions(); });
}

function reset() {
  console.log("Resetting CPU");
  emulator.cpu_reset();
  refreshRAMState();
  refreshRegisters();
  refreshInstructions();
}

function tick() {
  var cycles_used = emulator.cpu_tick();
  console.log("Ticked "+cycles_used+" cycles.");
}

function manualTick() {
  tick();
  refreshRAMState();
  refreshRegisters();
  refreshInstructions()
  refreshVRAM();
}

function autoTick() {
  if (shouldAutoTick.value == true) { tick(); }
}

function toggleAutoTick() {
  shouldAutoTick.value = !shouldAutoTick.value
}

function makeHex(value: number, hexlen: number) {
  return "0x"+value.toString(16).toUpperCase().padStart(hexlen,'0');
}

async function refreshRAMState() {
  console.log("Refreshing CPU memory...");
  currRAM.value=[];
}

async function refreshVRAM() {
  let ramSize = emulator.cpu_get_memory_size(); 
  console.log("Refreshing VRAM... size is: "+ramSize);
}

async function refreshRegisters() {
  // (&self.pc, &self.sp, &self.a, &self.b, &self.c, &self.d, &self.e, &self.h, &self.l)
  console.log("Refreshing Registers...");
  currRegisters.value=[];
  const regState = JSON.parse(emulator.cpu_registers());

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

async function refreshInstructions() {
  console.log("Refreshing instructions...");

  let instrs = emulator.cpu_instructions();
  let curr = instrs[0];
  let next = instrs[1];

  instructions.value=[];
  instructions.value.push({type:"C", opcode:makeHex(curr.opcode,2), size:curr.size, cycles:curr.cycles, text:curr.text});
  instructions.value.push({type:"N", opcode:makeHex(next.opcode,2), size:next.size, cycles:next.cycles, text:next.text});
}

async function run() {
  await init();
  emulator = new Emulator();

  sleep(500).then(() => { loadROM(); refreshVRAM(); });
}
run();

// Things we want to do after we've mounted.
onMounted(async () => {
  let pollInterval = setInterval(autoTick, 1) 
})

</script>

<template>
  <v-app id="8080">

    <v-navigation-drawer expand-on-hover rail>
      <v-list-item prepend-icon="mdi-robot-love-outline" title="8080"/>
      <v-divider></v-divider>
      <v-list density="compact" nav>
        <v-list-item prepend-icon="mdi-play-circle-outline" title="Load ROM" @click="loadROM"></v-list-item>
        <v-list-item prepend-icon="mdi-restart" title="Reset CPU" @click="reset()"></v-list-item>          
        <v-list-item prepend-icon="mdi-bug" title="Tick" @click="manualTick()"></v-list-item>
        <v-list-item v-if="shouldAutoTick" prepend-icon="mdi-autorenew" title="Auto Ticking" @click="toggleAutoTick()"></v-list-item>
        <v-list-item v-else prepend-icon="mdi-autorenew-off" title="Not Auto Ticking" @click="toggleAutoTick()"></v-list-item>
        <v-list-item prepend-icon="mdi-memory" title="Refresh RAM" @click="refreshRAMState()"></v-list-item>
        <v-list-item prepend-icon="mdi-ab-testing" title="Refresh Registers" @click="refreshRegisters()"></v-list-item>
        <v-list-item prepend-icon="mdi-ab-testing" title="Refresh Instructions" @click="refreshInstructions()"></v-list-item>
        <v-list-item prepend-icon="mdi-monitor" title="Refresh Screen" @click="refreshVRAM()"></v-list-item>
      </v-list>
    </v-navigation-drawer>

    <v-navigation-drawer permanent style="min-width:300px"> 
      <Registers :currRegisters=currRegisters :instructions=instructions />
    </v-navigation-drawer>

    <v-main>
      <v-sheet rounded class="d-flex pt-4">
        <Display />
      </v-sheet>

      <v-sheet rounded border class="d-flex">
        <Memory :currRAM=currRAM />
      </v-sheet>
    </v-main>

  </v-app>
</template>

<style scoped>


</style>

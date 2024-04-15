<script setup lang="ts">
import init, { cpu_greet, cpu_set_disassemble, cpu_get_disassemble, cpu_memory_write, 
  cpu_get_memory, cpu_state, cpu_curr_instr, cpu_tick } from 'emulator'

defineProps(['currRegisters'])

const registerHeaders = [
  { title: 'Register', value: 'register' },
  { title: 'Value', value: 'value' },
]

const hideDefaultFooter = true;

</script>

<template>
      <!-- <v-sheet rounded border> -->
        <v-data-table 
          :headers="registerHeaders" 
          :items="currRegisters"
          density="compact">
          <template v-slot:bottom v-if="hideDefaultFooter"></template>
        </v-data-table>
      <!-- </v-sheet> -->
</template>
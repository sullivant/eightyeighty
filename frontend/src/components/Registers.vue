<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import init, { cpu_greet, cpu_set_disassemble, cpu_get_disassemble, cpu_memory_write, 
  cpu_get_memory, cpu_state, cpu_curr_instr, cpu_tick, get_all_registers, cpu_reset } from 'emulator'

defineProps(['currRegisters','cpuState','lastInstr','nextInstr'])

const registerHeaders = [
  { title: 'Register', value: 'register' },
  { title: 'Value', value: 'value' },
]

const hideDefaultFooter = true;

</script>

<template>
      <v-list density="compact" nav>
        <v-list-item
          prepend-icon="mdi-puzzle-outline"
          title="Details"
        />
        <v-divider/>
          Last: {{  lastInstr }}
        <v-divider/>
          Next: {{ nextInstr }}
        <v-divider/>
         State: {{ cpuState }}
        <v-divider/>
        <v-data-table 
          :headers="registerHeaders" 
          :items="currRegisters"
          density="compact">
          <template v-slot:bottom v-if="hideDefaultFooter"></template>
        </v-data-table>
      </v-list>

      <!-- <v-sheet rounded border> -->

      <!-- </v-sheet> -->
</template>
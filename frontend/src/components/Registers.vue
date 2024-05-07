<script setup lang="ts">
import { ref, reactive, computed } from 'vue'

defineProps(['currRegisters','instructions'])

const registerHeaders = [
  { title: 'Register', value: 'register' },
  { title: 'Value', value: 'value' },
]

const instructionHeaders = [
  { title: '', value: 'type'},
  { title: 'O', value: 'opcode' },
  { title: 'S', value: 'size' },
  { title: 'C', value: 'cycles' },
  { title: 'Text', value: 'text'},
]

const hideDefaultFooter = true;

</script>

<template>
  <v-data-table
    :headers="instructionHeaders"
    :items="instructions"
    density="compact">
    <template v-slot:bottom v-if="hideDefaultFooter"></template>
  </v-data-table>
  
  <v-data-table 
    :headers="registerHeaders" 
    :items="currRegisters"
    density="compact">
    <template v-slot:bottom v-if="hideDefaultFooter"></template>
  </v-data-table>
</template>
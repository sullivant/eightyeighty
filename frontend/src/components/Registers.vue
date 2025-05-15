<script setup lang="ts">
import { ref, reactive, computed, onMounted, nextTick } from 'vue'
import { useDateFormat, useTimestamp } from '@vueuse/core'
defineProps(['currRegisters','instructions'])

const registerHeaders = [
  { title: 'Register', value: 'register' },
  { title: 'Value', value: 'value' },
]

const instructionHeaders = [
  { title: '', value: 'type'},
  { title: 'Op', value: 'opcode' },
  { title: 'S', value: 'size' },
  { title: 'C', value: 'cycles' },
  { title: 'Text', value: 'text' },
]

const hideDefaultFooter = true;

const time = useTimestamp()
const computedTime = useDateFormat(time, 'YYYY-MM-DD HH:mm:ss SSS')

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
  <v-sheet class="d-flex pt-4 text-center">
    {{ computedTime }}
  </v-sheet>
</template>
<script setup lang="ts">
import { ref } from 'vue'

defineProps(['currRAM']);
const search = ref('');

const memoryHeaders = [
  { title: 'Address', value: 'address', sortable: false},
  { title: '0x0', value: '0x0' },
  { title: '0x1', value: '0x1' },
  { title: '0x2', value: '0x2' },
  { title: '0x3', value: '0x3' },
  { title: '0x4', value: '0x4' },
  { title: '0x5', value: '0x5' },
  { title: '0x6', value: '0x6' },
  { title: '0x7', value: '0x7' },
  { title: '0x8', value: '0x8' },
  { title: '0x9', value: '0x9' },
  { title: '0xA', value: '0xA' },
  { title: '0xB', value: '0xB' },
  { title: '0xC', value: '0xC' },
  { title: '0xD', value: '0xD' },
  { title: '0xE', value: '0xE' },
  { title: '0xF', value: '0xF' },
]

//TODO: Highlight PC's location.  Maybe SP too?

</script>

<template>
  <v-card title="" flat >
    <template v-slot:text>
      <v-text-field
        v-model="search"
        label="Search"
        prepend-inner-icon="mdi-magnify"
        variant="outlined"
        hide-details
        clearable
        single-line
      ></v-text-field>
    </template>
    
    <v-data-table 
      :headers="memoryHeaders" 
      :items="currRAM" 
      :items-per-page="16"
      :search="search" density="compact">
    </v-data-table>
  </v-card>
</template>
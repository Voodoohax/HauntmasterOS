<template>
  <div class="space-y-2">
    <div v-for="asset in assets" :key="asset.id"
         draggable="true"
         @dragstart="startDrag($event, asset)"
         class="p-3 bg-gray-800 rounded cursor-move hover:bg-gray-700 border border-gray-600">
      <img :src="asset.thumb" class="w-full h-20 object-cover rounded mb-2" />
      <div class="text-center">
        <p class="text-xs font-semibold">{{ asset.name }}</p>
        <p class="text-xs text-gray-400">{{ asset.type }}</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { defineEmits } from 'vue'

const emit = defineEmits(['dragstart'])

function startDrag(event, asset) {
  event.dataTransfer.effectAllowed = 'copy'
  event.dataTransfer.setData('asset-id', asset.id)
  emit('dragstart', asset)
}
</script>

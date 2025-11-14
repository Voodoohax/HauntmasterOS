<template>
  <div ref="container" 
       @drop="handleDrop"
       @dragover="handleDragOver"
       class="w-full h-full bg-black relative">
    <canvas ref="canvas"></canvas>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const container = ref(null)
const canvas = ref(null)

onMounted(() => {
  const ctx = canvas.value.getContext('2d')
  canvas.value.width = container.value.clientWidth
  canvas.value.height = container.value.clientHeight
})

function handleDrop(event) {
  event.preventDefault()
  const rect = container.value.getBoundingClientRect()
  const x = event.clientX - rect.left
  const y = event.clientY - rect.top
  
  // Emit drop event with coordinates
  this.$emit('drop', { x, y, assetId: event.dataTransfer.getData('asset-id') })
}

function handleDragOver(event) {
  event.preventDefault()
  event.dataTransfer.dropEffect = 'copy'
}
</script>

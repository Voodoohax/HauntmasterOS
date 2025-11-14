<template>
  <div class="min-h-screen bg-black text-orange-500 p-4">
    <header class="text-center mb-6">
      <h1 class="text-4xl">🎃 HAUNTMASTER SCENE COMPOSER 🎃</h1>
      <p class="text-sm">Drag assets to haunt your house</p>
    </header>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <!-- ASSET LIBRARY -->
      <div class="bg-gray-900 p-6 rounded-lg border border-orange-800">
        <h2 class="text-xl mb-4">🕸️ ASSET LIBRARY</h2>
        <div class="space-y-2 max-h-96 overflow-y-auto">
          <div v-for="asset in assets" :key="asset.id"
               @dragstart="startDrag($event, asset)"
               class="p-2 bg-gray-800 rounded cursor-move hover:bg-gray-700">
            <img :src="asset.thumb" class="w-full h-20 object-cover rounded" />
            <p class="text-xs truncate mt-1">{{ asset.name }}</p>
          </div>
        </div>
      </div>

      <!-- CANVAS -->
      <div class="lg:col-span-2">
        <div class="bg-gray-900 p-6 rounded-lg border border-orange-800">
          <div class="flex justify-between items-center mb-4">
            <h2 class="text-xl">🎬 HAUNT CANVAS (16:9)</h2>
            <div class="space-x-2">
              <button @click="playScene" class="bg-green-900 px-4 py-2 rounded">▶ PLAY</button>
              <button @click="clearScene" class="bg-red-900 px-4 py-2 rounded">🗑 CLEAR</button>
              <button @click="saveScene" class="bg-blue-900 px-4 py-2 rounded">💾 SAVE</button>
            </div>
          </div>
          <div ref="stageContainer" class="w-full h-96 bg-black rounded border border-gray-700 relative">
            <canvas ref="stage" class="absolute inset-0"></canvas>
            <div v-if="dragPreview" class="absolute pointer-events-none" :style="dragStyle">
              <img :src="dragPreview.thumb" :style="{ width: dragPreview.width + 'px', height: dragPreview.height + 'px' }" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- LAYER PANEL -->
    <div class="mt-6 bg-gray-900 p-6 rounded-lg border border-orange-800">
      <h2 class="text-xl mb-4">🎭 LAYERS</h2>
      <div class="space-y-2">
        <div v-for="(layer, index) in scene.layers" :key="layer.id"
             class="flex items-center p-2 bg-gray-800 rounded hover:bg-gray-700">
          <span class="text-xs w-8">{{ index + 1 }}</span>
          <img :src="layer.thumb" class="w-8 h-8 object-cover rounded mr-2" />
          <span class="text-xs flex-1">{{ layer.name }}</span>
          <div class="space-x-1">
            <input type="range" v-model="layer.opacity" min="0" max="1" step="0.1" class="w-16" />
            <span class="text-xs">{{ Math.round(layer.opacity * 100) }}%</span>
          </div>
          <button @click="removeLayer(index)" class="ml-2 text-red-400">×</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, reactive } from 'vue'
import { Stage, Layer, Rect, Text, Image } from 'vue-konva'
import 'konva/lib/shapes/Image'

const stageContainer = ref(null)
const stage = ref(null)
const scene = reactive({
  layers: []
})
const dragPreview = ref(null)
const dragStyle = ref({})

const assets = ref([
  { id: 'ghost', name: 'Ghost', thumb: '/thumbs/ghost.webp', type: 'image' },
  { id: 'fog', name: 'Fog', thumb: '/thumbs/fog.webp', type: 'video' },
  { id: 'pumpkin', name: 'Pumpkin', thumb: '/thumbs/pumpkin.webp', type: 'image' },
  { id: 'skeleton', name: 'Skeleton', thumb: '/thumbs/skeleton.webp', type: 'image' }
])

onMounted(() => {
  // Initialize Konva stage
  const container = stageContainer.value
  if (container) {
    const width = container.clientWidth
    const height = container.clientHeight
    stage.value = new Konva.Stage({
      container: container,
      width: width,
      height: height
    })
    
    const layer = new Konva.Layer()
    stage.value.add(layer)
    layer.draw()
  }
})

function startDrag(event, asset) {
  dragPreview.value = asset
  event.dataTransfer.effectAllowed = 'copy'
  event.dataTransfer.setData('asset-id', asset.id)
}

function handleDrop(event) {
  event.preventDefault()
  const rect = stageContainer.value.getBoundingClientRect()
  const x = event.clientX - rect.left
  const y = event.clientY - rect.top
  
  const assetId = event.dataTransfer.getData('asset-id')
  const asset = assets.value.find(a => a.id === assetId)
  if (asset && stage.value) {
    const layer = new Konva.Layer()
    const konvaImage = new Konva.Image({
      x: x - 50,
      y: y - 50,
      image: new window.Image(),
      width: 100,
      height: 100,
      draggable: true,
      name: asset.name
    })
    
    konvaImage.image.onload = () => {
      layer.add(konvaImage)
      stage.value.add(layer)
      layer.draw()
      
      // Add to scene state
      scene.layers.push({
        id: Date.now(),
        name: asset.name,
        thumb: asset.thumb,
        opacity: 1.0,
        x: x - 50,
        y: y - 50,
        width: 100,
        height: 100,
        konvaImage: konvaImage
      })
    }
    
    konvaImage.image.src = asset.thumb
  }
  
  dragPreview.value = null
}

function handleDragOver(event) {
  event.preventDefault()
  event.dataTransfer.dropEffect = 'copy'
}

function playScene() {
  // TODO: Export scene to FFmpeg overlay command
  console.log('Playing scene:', scene.layers)
  // Send to backend API
  fetch('/api/scene/play', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      layers: scene.layers.map(l => ({
        path: l.thumb,
        x: l.x,
        y: l.y,
        width: l.width,
        height: l.height,
        opacity: l.opacity
      }))
    })
  })
}

function clearScene() {
  scene.layers = []
  if (stage.value) {
    stage.value.destroyChildren()
    stage.value.draw()
  }
}

function saveScene() {
  const sceneData = {
    name: `Haunt Scene ${new Date().toISOString().split('T')[0]}`,
    layers: scene.layers.map(l => ({
      name: l.name,
      path: l.thumb,
      x: l.x,
      y: l.y,
      width: l.width,
      height: l.height,
      opacity: l.opacity
    }))
  }
  
  fetch('/api/scene/save', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(sceneData)
  }).then(res => res.json()).then(data => {
    alert(`Scene saved: ${data.id}`)
  })
}

function removeLayer(index) {
  scene.layers.splice(index, 1)
  // TODO: Remove from Konva stage
}
</script>

<style scoped>
canvas {
  cursor: crosshair;
}
</style>

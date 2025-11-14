<template>
  <div class="min-h-screen bg-black text-orange-500 p-4">
    <header class="text-center mb-6">
      <h1 class="text-4xl">HAUNTMASTER SCENE COMPOSER</h1>
      <p class="text-sm">Drag, crop, layer, haunt</p>
    </header>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <!-- ASSET LIBRARY -->
      <div class="bg-gray-900 p-6 rounded-lg border border-orange-800">
        <h2 class="text-xl mb-4">ASSET LIBRARY</h2>
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
            <h2 class="text-xl">HAUNT CANVAS (16:9)</h2>
            <div class="space-x-2">
              <button @click="playScene" class="bg-green-900 px-4 py-2 rounded">PLAY</button>
              <button @click="clearScene" class="bg-red-900 px-4 py-2 rounded">CLEAR</button>
            </div>
          </div>
          <div ref="stageContainer" class="w-full h-96 bg-black rounded border border-gray-700 relative overflow-hidden">
            <canvas ref="stage" class="absolute inset-0"></canvas>
            <div v-if="selectedLayer" class="absolute border-2 border-orange-500 pointer-events-none"
                 :style="{ left: selectedCrop.x + 'px', top: selectedCrop.y + 'px', width: selectedCrop.w + 'px', height: selectedCrop.h + 'px' }">
              <div class="absolute top-0 left-0 w-3 h-3 bg-orange-500 cursor-nw-resize"></div>
              <div class="absolute top-0 right-0 w-3 h-3 bg-orange-500 cursor-ne-resize"></div>
              <div class="absolute bottom-0 left-0 w-3 h-3 bg-orange-500 cursor-sw-resize"></div>
              <div class="absolute bottom-0 right-0 w-3 h-3 bg-orange-500 cursor-se-resize"></div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- LAYERS -->
    <div class="mt-6 bg-gray-900 p-6 rounded-lg border border-orange-800">
      <h2 class="text-xl mb-4">LAYERS</h2>
      <div class="space-y-2">
        <div v-for="(layer, index) in scene.layers" :key="layer.id"
             @click="selectLayer(index)"
             class="flex items-center p-2 bg-gray-800 rounded hover:bg-gray-700 cursor-pointer"
             :class="{ 'ring-2 ring-orange-500': selectedIndex === index }">
          <span class="text-xs w-8">{{ index + 1 }}</span>
          <img :src="layer.thumb" class="w-8 h-8 object-cover rounded mr-2" />
          <span class="text-xs flex-1">{{ layer.name }}</span>
          <input type="range" v-model="layer.opacity" min="0" max="1" step="0.1" class="w-16" />
          <span class="text-xs w-12">{{ Math.round(layer.opacity * 100) }}%</span>
          <button @click.stop="removeLayer(index)" class="ml-2 text-red-400">×</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import Konva from 'konva'

const stageContainer = ref(null)
const stage = ref(null)
const scene = reactive({ layers: [] })
const selectedIndex = ref(null)
const selectedCrop = reactive({ x: 0, y: 0, w: 100, h: 100 })

const assets = ref([])

onMounted(async () => {
  const res = await fetch('/api/media')
  const media = await res.json()
  assets.value = media.map(m => ({
    id: m.id,
    name: m.name,
    thumb: m.thumb,
    path: m.path
  }))

  const width = stageContainer.value.clientWidth
  const height = stageContainer.value.clientHeight
  stage.value = new Konva.Stage({
    container: stageContainer.value,
    width, height
  })
  const layer = new Konva.Layer()
  stage.value.add(layer)
})

function startDrag(e, asset) {
  e.dataTransfer.setData('asset', JSON.stringify(asset))
}

function handleDrop(e) {
  e.preventDefault()
  const rect = stageContainer.value.getBoundingClientRect()
  const x = e.clientX - rect.left - 50
  const y = e.clientY - rect.top - 50
  const asset = JSON.parse(e.dataTransfer.getData('asset'))

  const imageObj = new Image()
  imageObj.onload = () => {
    const konvaImage = new Konva.Image({
      x, y, image: imageObj, width: 100, height: 100, draggable: true
    })
    const layer = new Konva.Layer()
    layer.add(konvaImage)
    stage.value.add(layer)

    scene.layers.push({
      id: Date.now(),
      name: asset.name,
      path: asset.path,
      thumb: asset.thumb,
      x, y, width: 100, height: 100,
      opacity: 1.0,
      crop: null,
      konvaImage
    })
  }
  imageObj.src = asset.thumb
}

function selectLayer(index) {
  selectedIndex.value = index
  const l = scene.layers[index]
  selectedCrop.x = l.x
  selectedCrop.y = l.y
  selectedCrop.w = l.width
  selectedCrop.h = l.height
}

function playScene() {
  fetch('/api/play', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ scene })
  })
}

function clearScene() {
  scene.layers = []
  stage.value.destroyChildren()
}

function removeLayer(i) {
  scene.layers.splice(i, 1)
}
</script>

<style>
#stageContainer { touch-action: none; }
</style>

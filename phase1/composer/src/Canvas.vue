<template>
  <div
    ref="container"
    @drop="handleDrop"
    @dragover="handleDragOver"
    @dragenter.prevent
    @dragleave.prevent
    class="relative w-full h-full bg-gray-950 overflow-hidden"
    :style="{ cursor: isDragging ? 'copy' : 'default' }"
  >
    <!-- KONVA STAGE -->
    <div ref="stageEl" class="absolute inset-0"></div>

    <!-- CROP OVERLAY (when layer selected) -->
    <div
      v-if="selectedLayer && isCropping"
      class="absolute border-2 border-orange-500 pointer-events-auto"
      :style="cropBoxStyle"
      @mousedown.stop="startCropDrag"
    >
      <!-- Crop Handles -->
      <div
        v-for="handle in cropHandles"
        :key="handle"
        class="absolute w-3 h-3 bg-orange-500 border border-white"
        :class="handleClasses[handle]"
        :style="handleStyles[handle]"
        @mousedown.stop="startCropResize($event, handle)"
      ></div>
    </div>

    <!-- DRAG PREVIEW -->
    <div
      v-if="dragPreview"
      class="absolute pointer-events-none z-50"
      :style="dragPreviewStyle"
    >
      <img
        :src="dragPreview.thumb"
        :style="{ width: dragPreview.width + 'px', height: dragPreview.height + 'px', opacity: 0.7 }"
        class="border-2 border-dashed border-orange-500 rounded"
      />
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, onUnmounted, computed } from 'vue'
import Konva from 'konva'

const props = defineProps({
  assets: Array,
  scene: Object,
  selectedIndex: Number
})

const emit = defineEmits(['layer-added', 'layer-updated', 'layer-selected'])

const container = ref(null)
const stageEl = ref(null)
const stage = ref(null)
const isDragging = ref(false)
const dragPreview = ref(null)
const dragPreviewStyle = ref({})

const selectedLayer = computed(() => 
  props.selectedIndex !== null ? props.scene.layers[props.selectedIndex] : null
)

const isCropping = ref(false)
const cropBox = reactive({ x: 0, y: 0, w: 100, h: 100 })
const cropStart = reactive({ x: 0, y: 0 })
const cropHandle = ref(null)

const cropHandles = ['tl', 'tr', 'bl', 'br', 't', 'r', 'b', 'l']
const handleClasses = {
  tl: '-top-1.5 -left-1.5 cursor-nw-resize',
  tr: '-top-1.5 -right-1.5 cursor-ne-resize',
  bl: '-bottom-1.5 -left-1.5 cursor-sw-resize',
  br: '-bottom-1.5 -right-1.5 cursor-se-resize',
  t: '-top-1.5 left-1/2 -translate-x-1/2 cursor-n-resize',
  r: '-right-1.5 top-1/2 -translate-y-1/2 cursor-e-resize',
  b: '-bottom-1.5 left-1/2 -translate-x-1/2 cursor-s-resize',
  l: '-left-1.5 top-1/2 -translate-y-1/2 cursor-w-resize'
}

const cropBoxStyle = computed(() => ({
  left: `${cropBox.x}px`,
  top: `${cropBox.y}px`,
  width: `${cropBox.w}px`,
  height: `${cropBox.h}px`
}))

const handleStyles = computed(() => {
  const s = {}
  cropHandles.forEach(handle => {
    s[handle] = {}
  })
  return s
})

onMounted(() => {
  initStage()
  window.addEventListener('mouseup', stopCrop)
  window.addEventListener('mousemove', updateCrop)
})

onUnmounted(() => {
  window.removeEventListener('mouseup', stopCrop)
  window.removeEventListener('mousemove', updateCrop)
})

function initStage() {
  const width = container.value.clientWidth
  const height = container.value.clientHeight

  stage.value = new Konva.Stage({
    container: stageEl.value,
    width,
    height,
    draggable: false
  })

  // Background grid
  const bgLayer = new Konva.Layer()
  const gridSize = 20
  for (let i = 0; i < width / gridSize; i++) {
    bgLayer.add(new Konva.Line({
      points: [i * gridSize, 0, i * gridSize, height],
      stroke: '#1a1a1a',
      strokeWidth: 1
    }))
  }
  for (let i = 0; i < height / gridSize; i++) {
    bgLayer.add(new Konva.Line({
      points: [0, i * gridSize, width, i * gridSize],
      stroke: '#1a1a1a',
      strokeWidth: 1
    }))
  }
  stage.value.add(bgLayer)

  // Safe area (16:9)
  const safe = new Konva.Rect({
    x: 100, y: 100,
    width: width - 200, height: height - 200,
    stroke: '#333',
    strokeWidth: 2,
    dash: [10, 5]
  })
  bgLayer.add(safe)
}

function handleDragOver(e) {
  e.preventDefault()
  isDragging.value = true
}

function handleDrop(e) {
  e.preventDefault()
  isDragging.value = false
  dragPreview.value = null

  const rect = container.value.getBoundingClientRect()
  const x = e.clientX - rect.left
  const y = e.clientY - rect.top

  const assetData = e.dataTransfer.getData('asset')
  if (assetData) {
    const asset = JSON.parse(assetData)
    addLayer(asset, x, y)
  }
}

function startDragPreview(asset, e) {
  dragPreview.value = {
    ...asset,
    width: 100,
    height: 100
  }
  updateDragPreview(e)
}

function updateDragPreview(e) {
  if (!dragPreview.value) return
  const rect = container.value.getBoundingClientRect()
  dragPreviewStyle.value = {
    left: `${e.clientX - rect.left - 50}px`,
    top: `${e.clientY - rect.top - 50}px`
  }
}

function addLayer(asset, x, y) {
  const imageObj = new Image()
  imageObj.onload = () => {
    const konvaImage = new Konva.Image({
      x: x - 50,
      y: y - 50,
      image: imageObj,
      width: 100,
      height: 100,
      draggable: true,
      name: asset.name
    })

    konvaImage.on('dragmove', () => {
      const layer = props.scene.layers.find(l => l.konvaImage === konvaImage)
      if (layer) {
        layer.x = konvaImage.x()
        layer.y = konvaImage.y()
      }
    })

    konvaImage.on('click tap', () => {
      const index = props.scene.layers.findIndex(l => l.konvaImage === konvaImage)
      emit('layer-selected', index)
    })

    const layer = new Konva.Layer()
    layer.add(konvaImage)
    stage.value.add(layer)

    const newLayer = {
      id: Date.now(),
      name: asset.name,
      path: asset.path,
      thumb: asset.thumb,
      x: x - 50,
      y: y - 50,
      width: 100,
      height: 100,
      opacity: 1.0,
      crop: null,
      konvaImage
    }

    props.scene.layers.push(newLayer)
    emit('layer-added', newLayer)
  }
  imageObj.src = asset.thumb
}

// CROP LOGIC
function startCrop() {
  if (!selectedLayer.value) return
  isCropping.value = true
  const l = selectedLayer.value
  cropBox.x = l.x
  cropBox.y = l.y
  cropBox.w = l.width
  cropBox.h = l.height
}

function startCropDrag(e) {
  const rect = container.value.getBoundingClientRect()
  cropStart.x = e.clientX - rect.left - cropBox.x
  cropStart.y = e.clientY - rect.top - cropBox.y
}

function startCropResize(e, handle) {
  cropHandle.value = handle
  const rect = container.value.getBoundingClientRect()
  cropStart.x = e.clientX - rect.left
  cropStart.y = e.clientY - rect.top
}

function updateCrop(e) {
  if (!isCropping.value) return
  const rect = container.value.getBoundingClientRect()
  const mx = e.clientX - rect.left
  const my = e.clientY - rect.top

  if (cropHandle.value) {
    // Resize
    const dx = mx - cropStart.x
    const dy = my - cropStart.y

    if (cropHandle.value.includes('r')) cropBox.w += dx
    if (cropHandle.value.includes('l')) { cropBox.x += dx; cropBox.w -= dx }
    if (cropHandle.value.includes('b')) cropBox.h += dy
    if (cropHandle.value.includes('t')) { cropBox.y += dy; cropBox.h -= dy }

    cropBox.w = Math.max(20, cropBox.w)
    cropBox.h = Math.max(20, cropBox.h)
    cropStart.x = mx
    cropStart.y = my
  } else {
    // Move
    cropBox.x = mx - cropStart.x
    cropBox.y = my - cropStart.y
  }

  // Snap to grid
  const grid = 10
  cropBox.x = Math.round(cropBox.x / grid) * grid
  cropBox.y = Math.round(cropBox.y / grid) * grid
  cropBox.w = Math.round(cropBox.w / grid) * grid
  cropBox.h = Math.round(cropBox.h / grid) * grid
}

function stopCrop() {
  if (!isCropping.value || !selectedLayer.value) return
  isCropping.value = false

  const l = selectedLayer.value
  l.x = cropBox.x
  l.y = cropBox.y
  l.width = cropBox.w
  l.height = cropBox.h
  l.crop = { x: 0, y: 0, width: l.width, height: l.height }

  if (l.konvaImage) {
    l.konvaImage.x(cropBox.x)
    l.konvaImage.y(cropBox.y)
    l.konvaImage.width(cropBox.w)
    l.konvaImage.height(cropBox.h)
    l.konvaImage.getLayer().draw()
  }

  emit('layer-updated', selectedLayer.value)
  cropHandle.value = null
}

// Expose crop control
defineExpose({ startCrop })
</script>

<style scoped>
.crop-overlay {
  box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.5);
}
</style>

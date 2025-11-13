<template>
  <div class="min-h-screen bg-black text-orange-500 p-4">
    <header class="text-center mb-6">
      <h1 class="text-4xl">HAUNTMASTER ZERO</h1>
      <p>Status: <span class="text-green-400">IDLE</span></p>
    </header>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- MEDIA VAULT -->
      <div class="bg-gray-900 p-6 rounded-lg border border-orange-800">
        <h2 class="text-xl mb-4">MEDIA VAULT</h2>
        
        <div 
          class="border-2 border-dashed border-orange-600 rounded-lg p-8 text-center cursor-pointer mb-4"
          @drop.prevent="handleDrop"
          @dragover.prevent
          @click="$refs.fileInput.click()"
        >
          <p>DROP FILES HERE</p>
          <input type="file" multiple ref="fileInput" @change="uploadFiles" class="hidden" />
        </div>

        <input v-model="search" placeholder="Search..." class="w-full p-2 bg-gray-800 rounded mb-4" />

        <div class="grid grid-cols-2 gap-3 max-h-96 overflow-y-auto">
          <div v-for="file in filteredMedia" :key="file.id" class="bg-gray-800 p-3 rounded">
            <img :src="file.thumb" class="w-full h-24 object-cover rounded" />
            <p class="text-xs truncate">{{ file.name }}</p>
            <div class="flex gap-1 mt-1">
              <button @click="play(file)" class="flex-1 bg-green-900 text-xs p-1 rounded">PLAY</button>
              <button @click="remove(file)" class="bg-red-900 text-xs p-1 rounded">DELETE</button>
            </div>
          </div>
        </div>
      </div>

      <!-- SCARE PANEL -->
      <div class="bg-gray-900 p-6 rounded-lg border border-orange-800">
        <h2 class="text-xl mb-4">SCARE PANEL</h2>

        <div class="flex gap-2 mb-4">
          <label><input type="checkbox" v-model="hdmi" /> HDMI</label>
          <label><input type="checkbox" v-model="audio" /> Audio</label>
          <button @click="stopAll" class="ml-auto bg-red-800 px-3 py-1 rounded">KILL</button>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <button
            v-for="file in recent"
            :key="file.id"
            @click="play(file)"
            class="relative bg-gray-800 p-3 rounded overflow-hidden"
            :class="{ 'ring-4 ring-red-600': playing?.id === file.id }"
          >
            <img :src="file.thumb" class="w-full h-20 object-cover" />
            <p class="text-xs truncate">{{ file.name }}</p>
            <div v-if="playing?.id === file.id" class="absolute top-1 right-1 animate-pulse">LIVE</div>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'

const media = ref([])
const search = ref('')
const hdmi = ref(true)
const audio = ref(true)
const playing = ref(null)

const filteredMedia = computed(() => 
  media.value.filter(f => f.name.toLowerCase().includes(search.value.toLowerCase()))
)

const recent = computed(() => media.value.slice(0, 6))

async function fetchMedia() {
  const res = await fetch('/api/media')
  media.value = await res.json()
}

async function uploadFiles(e) {
  const files = e.target.files || e.dataTransfer.files
  const form = new FormData()
  for (let f of files) form.append('files', f)
  await fetch('/api/upload', { method: 'POST', body: form })
  fetchMedia()
}

function handleDrop(e) { uploadFiles(e) }

async function play(file) {
  await fetch('/api/play', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ id: file.id, outputs: { hdmi: hdmi.value, audio: audio.value } })
  })
}

async function stopAll() {
  await fetch('/api/stop', { method: 'POST' })
}

async function remove(file) {
  if (confirm(`Delete ${file.name}?`)) {
    await fetch(`/api/media/${file.id}`, { method: 'DELETE' })
    fetchMedia()
  }
}

onMounted(() => {
  fetchMedia()
  setInterval(fetchMedia, 5000)
})
</script>

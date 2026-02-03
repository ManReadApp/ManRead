<template>
  <div class="space-y-8 p-4 text-gray-100 min-h-screen">
    <div v-for="section in sections" :key="section.key" class="space-y-4">
      <h3 class="text-lg font-semibold text-gray-300">{{ section.title }}</h3>

      <div v-if="section.type === 'selector'" class="space-y-2">
        <div
            v-for="item in section.items"
            :key="item.value"
            @click="!item.disabled && (selectedValues[section.key].value = item.value)"
            :class="[
            'px-4 py-2 rounded-md border transition-all cursor-pointer bg-gray-800 border-gray-700',
            {
              'ring-2 ring-blue-500': selectedValues[section.key].value === item.value,
              'hover:bg-gray-700': !item.disabled,
              'opacity-50 cursor-not-allowed pointer-events-none': item.disabled
            }
          ]"
        >
          {{ item.name }}
        </div>
      </div>
      <div v-else-if="section.type === 'button'" @click="$emit(section.items[0].value)"
           class="px-4 py-2 rounded-md border transition-all cursor-pointer bg-gray-800 border-gray-700 space-y-2"
      >{{section.items[0].name}}
      </div>

      <!--TODO: better full screen selector -->
      <div v-else class="grid grid-cols-2 gap-4">
        <div
            v-for="item in section.items"
            :key="item.name"
            :class="[
            'px-4 py-2 rounded-md bg-gray-800 border border-gray-700',
            {'opacity-50': item.disabled}
          ]"
        >
          <div class="flex justify-between items-center mb-2"
               @click="() => item.disabled ? null :  (sliderConfig[item.name]?.unit ? null : imageSettings[item.name] = (imageSettings[item.name] === 0 ? 1 : 0))">
            <span class="text-gray-300">{{ item.name }}</span>
            <span v-if="sliderConfig[item.name]?.unit" class="text-blue-400 font-mono">
              {{ imageSettings[item.name] }}{{ sliderConfig[item.name]?.unit }}
            </span>
            <span class="text-blue-400 font-mono" v-else>{{ imageSettings[item.name] === 1 }}</span>
          </div>
          <input
              v-if="!item.disabled && sliderConfig[item.name]?.unit"
              type="range"
              v-model="imageSettings[item.name]"
              :min="sliderConfig[item.name]?.min"
              :max="sliderConfig[item.name]?.max"
              :step="sliderConfig[item.name]?.step"
              class="w-full accent-blue-500 bg-gray-700 rounded-lg"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
const sections = [
  {
    title: 'Reader Mode',
    key: 'readerMode',
    type: 'selector',
    items: [
      {name: 'Vertical Scrolling', value: 'v', disabled: false},
      {name: 'Horizontal Scrolling', value: 'h', disabled: true},
      {name: 'Single Page', value: 's', disabled: false},
      {name: 'Double Page', value: 'd', disabled: false}
    ]
  },
  {
    title: 'Sizing Mode',
    key: 'sizeMode',
    type: 'selector',
    items: [
      {name: 'Screen Percentage', value: '%', disabled: false},
      {name: 'PX limit', value: 'l', disabled: false},
      {name: 'PX margin', value: 'm', disabled: false},
    ]
  },
  {
    title: 'Open Sizing Interface',
    key: 'openSizing',
    type: 'button',
    items: [{name: 'Open', value: 'openSizing'}]
  },
  {
    title: 'Read direction',
    key: 'readDirection',
    type: 'selector',
    items: [
      {name: 'Left to Right', value: 'ltr', disabled: false},
      {name: 'Right to Left', value: 'rtl', disabled: false}
    ]
  },
  {
    title: 'Image Settings',
    key: 'imageSettings',
    type: 'slider',
    items: [
      {name: 'Blur', disabled: false},
      {name: 'Brightness', disabled: false},
      {name: 'Contrast', disabled: false},
      {name: 'Grayscale', disabled: false},
      {name: 'Hue-rotate', disabled: false},
      {name: 'Invert', disabled: false},
      {name: 'Saturate', disabled: false},
      {name: 'Sepia', disabled: false},
      {name: 'Denoise', disabled: true}
    ]
  }
];


const readerMode = defineModel('readerMode', {required: true})
const sizeMode = defineModel('sizeMode', {required: true})

const readDirection = defineModel('readDirection', {required: true})
const selectedValues = {
  readerMode,
  sizeMode,
  readDirection
}

const sliderConfig = {
  Blur: {min: 0, max: 10, step: 1, unit: 'px'},
  Brightness: {min: 0, max: 200, step: 1, unit: '%'},
  Contrast: {min: 0, max: 200, step: 1, unit: '%'},
  Grayscale: {min: 0, max: 100, step: 1, unit: '%'},
  'Hue-rotate': {min: 0, max: 360, step: 1, unit: 'deg'},
  Invert: {min: 0, max: 100, step: 1, unit: '%'},
  Saturate: {min: 0, max: 200, step: 1, unit: '%'},
  Sepia: {min: 0, max: 100, step: 1, unit: '%'},
  Denoise: {min: 0, max: 1, step: 1, unit: ''}
};

const imageSettings = defineModel('imageSettings', {required: true})
</script>
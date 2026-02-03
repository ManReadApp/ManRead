<template>
  <Listbox v-model="selected">
    <div class="relative mt-0.5">
      <ListboxButton
          class="z-1 relative w-full cursor-default rounded-md bg-white px-3 py-1.5 text-base text-gray-900 outline-1 outline-gray-300 focus:outline-2 focus:outline-indigo-600 focus:outline-offset-2 focus:ring-0 sm:text-sm text-left"
      >
        <span class="block truncate" :class="{'invisible': !selected?.length}">{{
            selected?.length ? selected : 'None'
          }}</span>
        <span class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
          <LucideChevronsUpDown
              class="h-5 w-5 text-gray-400"
              aria-hidden="true"
          />
        </span>
      </ListboxButton>

      <transition
          leave-active-class="transition duration-100 ease-in"
          leave-from-class="opacity-100"
          leave-to-class="opacity-0"
      >
        <ListboxOptions
            :class="[
              'absolute max-h-60 w-full overflow-auto rounded-md bg-white py-1 text-base shadow-lg ring-1 ring-black/5 focus:outline-none sm:text-sm z-10',
              openUpwards ? 'bottom-full mb-1' : 'mt-1'
            ]"
        >
          <ListboxOption
              v-slot="{ active, selected }"
              v-for="sug in suggestions"
              :key="sug"
              :value="sug"
              as="template"
          >
            <li
                :class="[
                active ? 'bg-amber-100 text-amber-900' : 'text-gray-900',
                'relative cursor-default select-none py-2 pl-3 pr-4',
              ]"
            >
              <span
                  :class="[
                  selected ? 'font-medium' : 'font-normal',
                  'block truncate',
                ]"
              >{{ sug }}</span>
            </li>
          </ListboxOption>
        </ListboxOptions>
      </transition>
    </div>
  </Listbox>
</template>

<script lang="ts" setup>
import {defineProps, withDefaults} from 'vue'
import {Listbox, ListboxButton, ListboxOption, ListboxOptions,} from '@headlessui/vue'

interface Props {
  suggestions: string[]
  openUpwards?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  openUpwards: false,
})

const selected = defineModel<string>()
</script>
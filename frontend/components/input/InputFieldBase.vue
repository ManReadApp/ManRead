<script lang="ts">
import {computed, ref, watch} from 'vue';

export default {
  props: {
    disabled: {type: Boolean, required: true},
    modelValue: {type: String, required: true},
    autoComplete: {type: String, required: true},
    openUpwards: {type: Boolean, required: false},
    openButton: {type: Boolean, required: false},
    label: {type: String, required: true},
    id: {type: String, required: true},
    type: {type: String, required: true},
    completions: {type: Array as () => string[], required: false},
  },
  emits: ['update:modelValue', 'click:label', 'pressed:enter'],
  setup(props, {emit}) {
    const open = ref(false);
    const selectedIndex = ref(0);
    const userIsTyping = ref(false);

    const filteredCompletions = computed(() => {
      if (!props.completions) return [];
      if (!props.modelValue  && !props.openButton) return [];
      const input = props.modelValue.toLowerCase();
      return props.completions.filter(c => c.toLowerCase().includes(input));
    });

    watch(() => props.modelValue, () => {
      if (userIsTyping.value) {
        open.value = true;
        selectedIndex.value = 0;
      }
    });

    const onInput = (event: Event) => {
      userIsTyping.value = true;
      const target = event.target as HTMLInputElement | HTMLTextAreaElement;
      emit('update:modelValue', props.type === 'file' ? (target as HTMLInputElement).files : target.value);
    };

    const onKeydown = (event: KeyboardEvent) => {
      if (!open.value || !filteredCompletions.value.length) {
        if (event.key === 'Enter') {
          emit('pressed:enter')
        }
        return;
      }

      if (event.key === 'ArrowDown') {
        event.preventDefault();
        selectedIndex.value = (selectedIndex.value + 1) % filteredCompletions.value.length;
      } else if (event.key === 'ArrowUp') {
        event.preventDefault();
        selectedIndex.value = (selectedIndex.value - 1 + filteredCompletions.value.length) % filteredCompletions.value.length;
      } else if (event.key === 'Enter' || event.key === 'Tab') {
        if (selectedIndex.value >= 0) {
          event.preventDefault();
          select(filteredCompletions.value[selectedIndex.value]);
        }
      } else if (event.key === 'Escape') {
        open.value = false;
      }
    };

    const select = (value: string) => {
      open.value = false;
      selectedIndex.value = 0;
      userIsTyping.value = false;
      emit('update:modelValue', value);
    };

    const toggleOpen = () => {
      if (filteredCompletions.value.length) {
        open.value = !open.value;
      }
    };

    return {
      userIsTyping,
      open,
      selectedIndex,
      filteredCompletions,
      onInput,
      onKeydown,
      select,
      toggleOpen,
    };
  },
};
</script>

<template>
  <div>
    <label
        class="block text-sm/6 font-medium text-gray-900"
        :for="id"
        @click="$emit('click:label')"
    >
      {{ label }}
    </label>
    <div class="mt-2">
      <slot name="no-flex"/>
      <div class="flex relative">
        <slot name="flex"/>
        <div class="relative grow">
          <div class="relative">
            <component
                :is="type === 'textarea' ? 'textarea' : 'input'"
                :autocomplete="type === 'textarea' ? undefined : autoComplete"
                class="block w-full rounded-md bg-white px-3 py-1.5 pr-10 text-base text-gray-900 outline-1 -outline-offset-1 outline-gray-300 placeholder:text-gray-400 focus:outline-2 focus:-outline-offset-2 focus:outline-indigo-600 sm:text-sm/6"
                :id="id"
                :name="id"
                :type="type !== 'textarea' ? type : undefined"
                :value="modelValue"
                @keydown="onKeydown"
                @input="onInput"
                :disabled="disabled"
                :accept="type === 'file' ? 'image/*' : undefined"
                required
            />

            <!-- Open Button (inside input) -->
            <button
                v-if="openButton"
                type="button"
                @click="toggleOpen"
                class="absolute inset-y-0 right-0 flex items-center px-2 text-gray-500 hover:text-gray-700"
                :disabled="disabled"
                tabindex="-1"
            >
              ▼
            </button>
          </div>

          <!-- Suggestions Dropdown -->
          <div
              v-if="open && filteredCompletions.length"
              :class="[
              'absolute w-full bg-white shadow-lg border border-gray-300 rounded-md max-h-60 overflow-auto z-10',
              openUpwards ? 'bottom-full mb-1' : 'top-full mt-1'
            ]"
          >
            <ul class="py-1 text-sm text-gray-700">
              <li
                  v-for="(completion, index) in filteredCompletions"
                  :key="completion"
                  :class="[
                  'px-4 py-2 cursor-pointer',
                  index === selectedIndex ? 'bg-indigo-600 text-white' : 'hover:bg-gray-100'
                ]"
                  @mousedown.prevent="select(completion)"
              >
                {{ completion }}
              </li>
            </ul>
          </div>

        </div>
      </div>
    </div>
  </div>
</template>
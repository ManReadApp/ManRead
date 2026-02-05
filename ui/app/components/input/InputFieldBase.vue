<script lang="ts" setup>
import { computed, ref, watch } from "vue";

const props = defineProps<{
    disabled: boolean;
    autoComplete: string;
    openUpwards: boolean;
    openButton: boolean;
    label: string;
    iid: string;
    type: string;
    completions?: string[];
}>();

const emits = defineEmits(["click:label", "pressed:enter"]);

const model = defineModel<string>();

const open = ref(false);
const selectedIndex = ref(0);
const userIsTyping = ref(false);

const filteredCompletions = computed(() => {
    if (!props.completions) return [];
    if (!model.value && !props.openButton) return [];
    const input = model.value!.toLowerCase();
    return props.completions.filter((c) => c.toLowerCase().includes(input));
});

watch(
    () => model.value,
    () => {
        if (userIsTyping.value) {
            open.value = true;
            selectedIndex.value = 0;
        }
    },
);

const onInput = (event: Event) => {
    userIsTyping.value = true;
    const target = event.target as HTMLInputElement | HTMLTextAreaElement;
    model.value =
        props.type === "file"
            ? (target as HTMLInputElement).files
            : target.value;
};

const onKeydown = (event: KeyboardEvent) => {
    if (!open.value || !filteredCompletions.value.length) {
        if (event.key === "Enter") {
            emits.apply("pressed:enter", "");
        }
        return;
    }

    if (event.key === "ArrowDown") {
        event.preventDefault();
        selectedIndex.value =
            (selectedIndex.value + 1) % filteredCompletions.value.length;
    } else if (event.key === "ArrowUp") {
        event.preventDefault();
        selectedIndex.value =
            (selectedIndex.value - 1 + filteredCompletions.value.length) %
            filteredCompletions.value.length;
    } else if (event.key === "Enter" || event.key === "Tab") {
        if (selectedIndex.value >= 0) {
            event.preventDefault();
            select(filteredCompletions.value[selectedIndex.value]);
        }
    } else if (event.key === "Escape") {
        open.value = false;
    }
};

const select = (value: string) => {
    open.value = false;
    selectedIndex.value = 0;
    userIsTyping.value = false;
    model.value = value;
};

const toggleOpen = () => {
    if (filteredCompletions.value.length) {
        open.value = !open.value;
    }
};
</script>

<template>
    <div>
        <label
            class="block text-sm/6 font-medium text-slate-700 dark:text-slate-200"
            :for="iid"
            @click="$emit('click:label')"
        >
            {{ label }}
        </label>
        <div class="mt-2">
            <slot name="no-flex" />
            <div class="flex relative">
                <slot name="flex" />
                <div class="relative grow">
                    <div class="relative">
                        <component
                            :is="type === 'textarea' ? 'textarea' : 'input'"
                            :id="iid"
                            :autocomplete="
                                type === 'textarea' ? undefined : autoComplete
                            "
                            class="block w-full rounded-md bg-white px-3 py-2 pr-10 text-base text-slate-900 ring-1 ring-slate-300/70 placeholder:text-slate-400 focus:ring-2 focus:ring-indigo-500 dark:bg-slate-900 dark:text-slate-100 dark:ring-slate-700 dark:focus:ring-indigo-400 sm:text-sm/6"
                            :name="iid"
                            :type="type !== 'textarea' ? type : undefined"
                            :value="modelValue"
                            :disabled="disabled"
                            :accept="type === 'file' ? 'image/*' : undefined"
                            required
                            @keydown="onKeydown"
                            @input="onInput"
                        />

                        <!-- Open Button (inside input) -->
                        <button
                            v-if="openButton"
                            type="button"
                            tabindex="-1"
                            class="absolute inset-y-0 right-0 flex items-center px-2 text-gray-500 hover:text-gray-700"
                            :disabled="disabled"
                            @click="toggleOpen"
                        >
                            ▼
                        </button>
                    </div>

                    <!-- Suggestions Dropdown -->
                    <div
                        v-if="open && filteredCompletions.length"
                        :class="[
                            'absolute z-10 w-full max-h-60 overflow-auto rounded-md border border-slate-200 bg-white shadow-lg dark:border-slate-700 dark:bg-slate-900',
                            openUpwards ? 'bottom-full mb-1' : 'top-full mt-1',
                        ]"
                    >
                        <ul class="py-1 text-sm text-slate-700 dark:text-slate-200">
                            <li
                                v-for="(
                                    completion, index
                                ) in filteredCompletions"
                                :key="completion"
                                :class="[
                                    'px-4 py-2 cursor-pointer',
                                    index === selectedIndex
                                        ? 'bg-indigo-600 text-white'
                                        : 'hover:bg-slate-100 dark:hover:bg-slate-800',
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

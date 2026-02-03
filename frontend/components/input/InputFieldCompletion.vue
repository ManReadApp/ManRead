<template>
    <Combobox v-model="selected">
        <div class="relative mt-1 z-10">
            <div>
                <ComboboxInput
                    class="block w-full rounded-md bg-white px-3 py-1.5 text-base text-gray-900 outline-1 -outline-offset-1 outline-gray-300 placeholder:text-gray-400 focus:outline-2 focus:-outline-offset-2 focus:outline-indigo-600 sm:text-sm/6"
                    :display-value="(item) => item"
                    @change="query = $event.target.value"
                />
            </div>
            <TransitionRoot
                leave="transition ease-in duration-100"
                leave-from="opacity-100"
                leave-to="opacity-0"
                @after-leave="query = selected"
            >
                <ComboboxOptions
                    v-if="filteredItems.length == 0"
                    class="absolute mt-1 max-h-60 w-full overflow-auto rounded-md bg-white py-1 text-base shadow-lg ring-1 ring-black/5 focus:outline-none sm:text-sm"
                >
                    <ComboboxOption
                        v-for="item in filteredItems"
                        :key="item"
                        v-slot="{ selected, active }"
                        as="template"
                        :value="item"
                    >
                        <li
                            class="relative cursor-default select-none py-2 pl-2 pr-4"
                            :class="{
                                'bg-teal-600 text-white': active,
                                'text-gray-900': !active,
                            }"
                        >
                            <span
                                class="block truncate"
                                :class="{
                                    'font-medium': selected,
                                    'font-normal': !selected,
                                }"
                            >
                                {{ item }}
                            </span>
                        </li>
                    </ComboboxOption>
                </ComboboxOptions>
            </TransitionRoot>
        </div>
    </Combobox>
</template>

<script setup>
import { computed, ref } from "vue";
import {
    Combobox,
    ComboboxInput,
    ComboboxOption,
    ComboboxOptions,
    TransitionRoot,
} from "@headlessui/vue";

const props = defineProps({
    items: {
        type: Array,
        required: true,
        default: () => [],
    },
});

const selected = ref(props.items[0] || "");
const query = defineModel();

const filteredItems = computed(() => {
    if (query.value === "") {
        return props.items;
    }

    const searchQuery = query.value.toLowerCase().replace(/\s+/g, "");
    return props.items.filter((item) => {
        let temp = item.toLowerCase().replace(/\s+/g, "");
        return temp.includes(searchQuery) && temp !== searchQuery;
    });
});
</script>

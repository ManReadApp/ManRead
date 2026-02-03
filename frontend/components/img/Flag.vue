<script setup lang="ts">
const props = defineProps({
  lang: {
    required: true,
    type: String
  },
  title: {
    required: true,
    type: String
  }
})

function calculateLatinRatio(str: string) {
  const allowedCharsRegex = /^[a-zA-Z0-9,.\-À-ÿ\s]+$/;

  const validChars = str.split('').filter(char => allowedCharsRegex.test(char));

  return validChars.length / str.length;
}

const is_converted = props.lang.endsWith("_latin")
let lang = props.lang;
if (is_converted) {
  lang = lang?.replace("_latin", "")
}
if (lang == "en") {
  lang = "us"
}

const is_ascii = calculateLatinRatio(props.title) > 0.6

</script>
<template>
  <div class="select-none"
       style="display: inline-block !important; min-width: 24px; min-height: 24px;"><img
      alt="flag" class="select-none"
      height="24" :src="`/icons/flags/4x3/${lang}.svg`"
      width="24">
    <img v-if="!is_ascii || is_converted"
         alt="script icon"
         height="12"
         :src="is_converted ? '/icons/scripts/latin.svg':'/icons/scripts/kanji.svg'"
         style="margin-top: -12px; margin-left: auto; margin-right: -2px;"
         width="12"></div>
</template>
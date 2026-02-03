<script lang="ts">
let listener = () => {
};
export default {
  emits: ['scroll'],
  mounted() {
    const container = document.querySelector("#scroll-container");
    if (container) {
      let lastScrollTop = 2000;
      let lastScrollLeft = 2000;

      container.scrollTo(2000, 2000);
      listener = () => {
        let deltaY = container.scrollTop - lastScrollTop;
        let deltaX = container.scrollLeft - lastScrollLeft;
        //TODO: why does the progress reset
        if (container.scrollTop > 50 && container.scrollLeft > 50) {
          this.$emit("scroll", [deltaX, deltaY]);
        }

        container.scrollTo(lastScrollLeft, lastScrollTop);

        lastScrollTop = container.scrollTop;
        lastScrollLeft = container.scrollLeft;
      };
      container.addEventListener("scroll", listener);
    }
  },
  beforeUnmount() {
    const container = document.querySelector("#scroll-container");
    if (container) {
      container.removeEventListener('scroll', listener)
    }
  }
}

</script>

<template>
  <div class="absolute top-0 left-0 w-full h-full overflow-hidden">
    <div ref="scroll_container" id="scroll-container" class="relative w-full h-full">
      <div id="scroll-area"/>
    </div>
  </div>
</template>

<style scoped>
#scroll-container {
  scroll-behavior: auto;
  overflow: scroll;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

#scroll-area {
  position: absolute;
  top: 0;
  left: 0;
  width: 4000px;
  height: 4000px;
}
</style>
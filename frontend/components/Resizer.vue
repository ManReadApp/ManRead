<script lang="ts">
import {
  calculatePercentage,
  calculatePxValueMax,
  calculatePxValueReduce,
  calculateSliderPercentage,
  calculateSliderReduce,
  calculateSliderValueMax
} from "~/utils/resizer";

export default {

  props: {
    inputHorizontal: {
      type: Number,
      required: true,
    },
    inputVertical: {
      type: Number,
      required: true,
    },
    mode: {
      type: String as () => 'l' | '%' | 'm',
      required: true,
      validator(value: number) {
        return [0, 1, 2].includes(value);
      }
    }
  },
  setup(props) {
    //TODO: watchEffect in case props.inputHorizontal or props.inputVertical are changed outside of this component
    switch (props.mode) {
      case '%':
        return {
          horizontal: ref(calculateSliderPercentage(props.inputHorizontal.valueOf())),
          vertical: ref(calculateSliderPercentage(props.inputVertical.valueOf())),
        }
      case 'm':
        return {
          horizontal: ref(calculateSliderReduce(window.innerWidth, props.inputHorizontal.valueOf())),
          vertical: ref(calculateSliderReduce(window.innerWidth, props.inputVertical.valueOf())),
        }
      case 'l':
        return {
          horizontal: ref(calculateSliderValueMax(window.innerWidth, props.inputHorizontal.valueOf())),
          vertical: ref(calculateSliderValueMax(window.innerWidth, props.inputVertical.valueOf())),
        }
      default:
        return {
          horizontal: ref(25),
          vertical: ref(25)
        }
    }

  },
  mounted() {
    window.addEventListener("resize", this.handleResize);
  },
  beforeUnmount() {
    window.removeEventListener("resize", this.handleResize);
  },
  emits: ['reduce-width', 'reduce-height', 'max-width', 'max-height', 'percentage-height', 'percentage-width'],
  methods: {
    handleResize() {
      switch (this.mode) {
        case 'm':
          this.horizontal = calculateSliderReduce(window.innerWidth, this.inputHorizontal.valueOf());
          this.vertical = calculateSliderReduce(window.innerHeight, this.inputVertical.valueOf());
          break;
        case 'l':
          this.horizontal = calculateSliderValueMax(window.innerWidth, this.inputHorizontal.valueOf());
          this.vertical = calculateSliderValueMax(window.innerHeight, this.inputVertical.valueOf());
          break;
      }
    }
  },
  watch: {
    horizontal: {
      immediate: true,
      handler(newHorizontal) {
        switch (this.mode) {
          case '%':
            const percentage = calculatePercentage(newHorizontal);
            this.$emit('percentage-width', percentage);
            break;
          case 'm':
            const px3 = calculatePxValueReduce(window.innerWidth, newHorizontal);
            this.$emit('reduce-width', px3);
            break;
          case 'l':
            const px4 = calculatePxValueMax(window.innerWidth, newHorizontal);
            this.$emit('max-width', px4);
            break
        }
      }
    },
    vertical: {
      immediate: true,
      handler(newVertical) {
        switch (this.mode) {
          case '%':
            const percentage = calculatePercentage(newVertical);
            this.$emit('percentage-height', percentage);
            break;
          case 'm':
            const px1 = calculatePxValueReduce(window.innerHeight, newVertical);
            this.$emit('reduce-height', px1);
            break;
          case 'l':
            const px2 = calculatePxValueMax(window.innerHeight, newVertical);
            this.$emit('max-height', px2);
            break
        }
      }
    }
  }
}

</script>

<template>
  <div class="absolute inset-0">
    <input type="range" class="slider-h horizontal-left" min="0" max="50" v-model="horizontal" step="0.001"/>
    <input type="range" class="slider-v vertical-top" min="0" max="50" v-model="vertical" step="0.001"/>
    <input type="range" class="slider-h horizontal-right" min="0" max="50" v-model="horizontal" step="0.001"/>
    <input type="range" class="slider-v vertical-bottom" min="0" max="50" v-model="vertical" step="0.001"/>
  </div>
</template>

<style scoped>
.slider-v {
  position: absolute;
  top: 0;
  left: 0;
  height: 20px;
  width: 50vh;
}

.slider-h {
  position: absolute;
  top: 0;
  left: 0;
  height: 20px;
  width: 50vw;
}

.vertical-top {
  /*transform: rotate(270deg) translateY(calc(-25vh + 50vw)) translateX(calc(-25vh + 10px));*/
  transform: rotate(90deg) translateY(calc(25vh - 50vw)) translateX(calc(25vh - 10px));
}

.horizontal-right {
  /* transform: translateX(100%) translateY(calc(50vh - 50%)); */
  transform: rotate(180deg) translateX(-100%) translateY(calc(-50vh + 50%));
}

.vertical-bottom {
  /*transform: rotate(90deg) translateY(calc(25vh - 50vw)) translateX(calc(50vh + 25vh - 10px));*/
  transform: rotate(270deg) translateY(calc(-25vh + 50vw)) translateX(calc(-50vh - 25vh + 10px));
}

.horizontal-left {
  /*transform: rotate(180deg) translateY(calc(-50vh + 50%));*/
  transform: translateY(calc(50vh - 50%));
}

</style>
<script lang="ts">
import { getCoverUrl } from "~/utils/cover";

export default {
    props: {
        classList: {
            type: String,
            default: "",
        },
        status: {
            type: String,
            enum: ["Dropped", "Hiatus", "Ongoing", "Completed", "Upcoming"],
            required: true,
        },
        manga_id: {
            type: String,
            required: true,
        },
        ext: {
            type: String,
            required: true,
        },
    },
    methods: {
        getColor():
            | "#de3b3b"
            | "#DA7500"
            | "#00C9F5"
            | "#7D40FF"
            | "#04D000"
            | "#ffffff" {
            switch (this.status) {
                case "Dropped":
                    return "#de3b3b";
                case "Hiatus":
                    return "#DA7500";
                case "Completed":
                    return "#00C9F5";
                case "Upcoming":
                    return "#7D40FF";
                case "Ongoing":
                    return "#04D000";
                default:
                    return "#ffffff";
            }
        },
    },
    computed: {
        bgColor() {
            const color = this.getColor();
            return `background: ${color};`;
        },
        generateLink(): string {
            return getCoverUrl(this.manga_id, this.ext);
        },
    },
};
</script>

<template>
    <div class="relative overflow-hidden" :class="classList">
        <img :src="generateLink" alt="" class="w-full h-full" />
        <span
            v-if="status == 'Dropped' || status == 'Hiatus'"
            class="status"
            :style="bgColor"
            >{{ status.toUpperCase() }}</span
        >
        <span v-else class="dot" :style="bgColor"></span>
    </div>
</template>

<style scoped>
.status {
    position: absolute;
    top: 20px;
    left: -22px;
    width: 100px;
    line-height: normal;
    color: #fff;
    text-transform: uppercase;
    text-align: center;
    font-weight: 700;
    z-index: 1;
    padding: 2px 18px;
    font-size: 9px;
    transform: rotate(-45deg);
}

.dot {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 9px;
    height: 9px;
    border-radius: 100%;
    border: white 1px solid;
}
</style>

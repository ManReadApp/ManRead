<script lang="ts" setup>
function getCoverUrl(manga_id: string, ext: string) {
    return `/api/v1/image-no-auth/cover/${manga_id}.${ext === "qoi" ? "avif" : ext}`;
}
const props = defineProps<{
    classList: string;
    status: string;
    mangaId: string;
    ext: string;
}>();

function getColor():
    | "#de3b3b"
    | "#DA7500"
    | "#00C9F5"
    | "#7D40FF"
    | "#04D000"
    | "#ffffff" {
    switch (props.status) {
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
}

const bgColor = computed(() => {
    return `background: ${getColor()};`;
});
const generateLink = computed(() => {
    return getCoverUrl(props.mangaId, props.ext);
});
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
        <span v-else class="dot" :style="bgColor" />
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

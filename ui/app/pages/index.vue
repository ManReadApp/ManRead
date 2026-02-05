<template>
    <AuthGuard>
        <div class="flex h-full w-full">
            <div class="h-full overflow-y-auto hide-scroll">
                <div v-if="value" class="m-8">
                    <HomeBar
                        v-if="value.reading.length != 0"
                        title="Reading"
                        :data="value.reading"
                        route="/search?desc=true&order=last_read"
                        :no-more="false"
                    />
                    <HomeBar
                        v-if="value.trending.length != 0"
                        title="Trending"
                        :data="value.trending"
                        route="/search?desc=false&order=trending"
                        :no-more="false"
                    />
                    <HomeBar
                        v-if="value.favorites.length != 0"
                        title="Favorites"
                        :data="value.favorites"
                        route="/search?desc=false&order=alphabetical&query=list:favorites"
                        :no-more="false"
                    />
                    <HomeBar
                        v-if="value.newest.length != 0"
                        title="Newest"
                        :data="value.newest"
                        route="/search?desc=true&order=created"
                        :no-more="false"
                    />
                    <HomeBar
                        v-if="value.latest_updates.length != 0"
                        title="Latest Updates"
                        :data="value.latest_updates"
                        route="/search?desc=true&order=updated"
                        :no-more="false"
                    />
                    <HomeBar
                        v-if="value.random.length != 0"
                        title="Random"
                        :data="value.random"
                        route="/"
                        :no-more="true"
                    />
                </div>
            </div>
            <Sidebar />
        </div>

        <!--
    TODO: https://www.npmjs.com/package/bottom-navigation-vue
    <WindowsBottomNavigation
        background-color='#FFFFFF'
        border-color='#9B9B9B'
        badge-color='#828282'
    ></WindowsBottomNavigation> -->
    </AuthGuard>
</template>

<script setup lang="ts">
import HomeBar from "~/components/home/HomeBar.vue";
import Sidebar from "~/components/sidebar/SideBar.vue";
useHead({
    title: "Home - ManRead",
});
const { $manRead } = useNuxtApp();
const { data: value } = await useAsyncData("home-data", async () => {
    return await $manRead("/api/v1/manga/home", {
        method: "POST",
        headers: { Authorization: `Bearer ${await getAccessToken()}` },
    });
});
</script>
<style>
.hide-scroll::-webkit-scrollbar {
    display: none;
}

.hide-scroll {
    scrollbar-width: none;
}
</style>

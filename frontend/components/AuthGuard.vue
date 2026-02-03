<template>
  <slot v-if="hasAccess"/>
</template>

<script setup lang="ts">
import {getAccessToken} from "~/utils/auth.js";
import {type Claim, useClaim} from "~/composables/useClaim.js";
import {navigateTo, onMounted} from "#imports";

const props = defineProps({
  roles: {
    type: Array as () => string[],
    default: [],
  },
});

const hasAccess = ref(false);

await useAsyncData("claim", async () => {
  if (!import.meta.server) return -1;
  await getAccessToken();
  return null;
});

const {subscribe} = useClaim();

const checkAccess = (c: Claim | null) => {
  if (c != null && (props.roles.length === 0 || props.roles.includes(c!.role))) {
    if (c!.role === "NotVerified") {
      hasAccess.value = false;
      navigateTo("/verify");
    } else {
      hasAccess.value = true;
    }
  } else {
    hasAccess.value = false;
    navigateTo("/sign-in");
  }
};


onMounted(async () => {
  const claimState = useState<Claim | null>("claim");
  if (!claimState.value) {
    await getAccessToken();
  }
  checkAccess(claimState.value);
  subscribe(checkAccess);
});
</script>

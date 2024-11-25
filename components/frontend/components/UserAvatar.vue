<script setup lang="ts">
const colors = [
    "#40407a",
    "#706fd3",
    "#34ace0",
    "#227093",
    "#218c74",
    "#ff5252",
    "#ff793f",
    "#ffb142",
    "#b33939",
    "#84817a",
    "#cc8e35",
    "#ccae62"
];
const { fontSize = '.7rem', size = '1.5rem', showTooltip = true, user } =
    defineProps<{ fontSize?: string, size?: string, showTooltip?: boolean, user?: { firstName: string, lastName: string } }>();

const auth = await useAuth();
const userObj = computed(() => user ?? auth.userProfile.user);
const color = computed(() => colors[(userObj.value.firstName.length + userObj.value.lastName.length) % colors.length]);
</script>

<template>
    <div class="d-flex align-items-center justify-content-center"
        v-b-tooltip.hover.top="showTooltip ? `${userObj.firstName} ${userObj.lastName}` : undefined"
        :style="{ height: size, width: size, borderRadius: '50%', backgroundColor: color, color: 'white', fontSize: fontSize }">
        {{ userObj.firstName[0] }}{{ userObj.lastName[0] }}
    </div>
</template>
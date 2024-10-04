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
const props = withDefaults(defineProps<{ firstName?: string, lastName?: string, size: string, fontSize: string }>(), {
    fontSize: '.7rem',
    size: '1.5rem'
})
const auth = useAuthMandatory();
const firstName = computed(() => props.firstName ?? auth.userProfile.user.firstName);
const lastName = computed(() => props.lastName ?? auth.userProfile.user.lastName);
const color = computed(() => colors[(firstName.value.length + lastName.value.length) % colors.length]);
</script>

<template>
    <div class="d-flex align-items-center justify-content-center"
        :style="{ height: props.size, width: props.size, borderRadius: '50%', backgroundColor: color, color: 'white', fontSize: props.fontSize }">
        {{ firstName[0] }}{{ lastName[0] }}
    </div>
</template>
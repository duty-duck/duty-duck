<script setup lang="ts">
import type { User } from 'bindings/User';

const user = defineModel<User | null>();

</script>


<template>
    <BModal title="Revoke user from organization" :model-value="!!user" @update:model-value="user = null">
        foo
    </BModal>
</template>
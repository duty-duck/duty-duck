<script setup lang="ts">
import type { ListOrganizationMembersParams } from "bindings/ListOrganizationMembersParams";
import type { ListOrganizationMembersItem } from "bindings/ListOrganizationMembersItem";
import { useOrganizationRepository } from "~/composables/useOrganizationRepository";
const localePath = useLocalePath();
const orgRepo = useOrganizationRepository();
const route = useRoute();
const auth = useAuthMandatory();

const pageNumber = computed(() => {
    return route.query.page ? Number(route.query.page) : 1
});

const fetchParams = ref<ListOrganizationMembersParams>({
    pageNumber: pageNumber.value,
    itemsPerPage: 20,
});

const { data: organizationMembers } = await orgRepo.useOrganizationMembers(fetchParams);

const revokeUserModal = ref<ListOrganizationMembersItem | null>(null);
const updateRolesModal = ref<ListOrganizationMembersItem | null>(null);
</script>

<template>
    <BContainer>
        <OrganizationRevokeUserModal v-model="revokeUserModal" />
        <OrganizationUpdateUserRolesModal v-model="updateRolesModal" />
        <BBreadcrumb>
            <BBreadcrumbItem :to="localePath('/dashboard')">{{ $t("dashboard.sidebar.home") }}</BBreadcrumbItem>
            <BBreadcrumbItem active>{{ $t("dashboard.myOrg.pageTitle") }}</BBreadcrumbItem>
        </BBreadcrumb>
        <h2>
            <Icon name="ph:users" />
            {{ $t("dashboard.myOrg.pageTitle") }}
        </h2>
        <BCard :title="$t('dashboard.myOrg.members.title')">
            <BTableSimple v-if="organizationMembers" class="users-table" responsive>
                <BThead>
                    <BTr>
                        <BTh>{{ $t("dashboard.myOrg.members.name") }}</BTh>
                        <BTh>{{ $t("dashboard.myOrg.members.email") }}</BTh>
                        <BTh>{{ $t("dashboard.myOrg.members.role") }}</BTh>
                        <BTh>{{ $t("dashboard.myOrg.members.manage") }}</BTh>
                    </BTr>
                </BThead>
                <BTbody>
                    <BTr v-for="member in organizationMembers.members" :key="member.id">
                        <BTd>{{ member.firstName }} {{ member.lastName }}
                            <span v-if="member.id == auth.userProfile.user.id">{{ $t("dashboard.myOrg.members.yourself") }}</span>
                        </BTd>
                        <BTd>{{ member.email }}</BTd>
                        <BTd>
                            <BBadge v-for="role in member.organizationRoles" :key="role">{{ role }}</BBadge>
                        </BTd>
                        <BTd>
                            <div v-if="member.organizationRoles.includes('Owner')" class="text-muted">
                                {{ $t("dashboard.myOrg.members.cannotEditOwner") }}
                            </div>
                            <div class="d-flex gap-2" v-else>
                                <BButton pill variant="outline-danger" size="sm"
                                    @click="revokeUserModal = member; updateRolesModal = null"
                                    v-if="auth.can('removeOrganizationMember') && member.id != auth.userProfile.user.id">
                                    {{ $t("dashboard.myOrg.members.revokeUser") }}
                                </BButton>
                                <BButton pill variant="outline-secondary" size="sm"
                                    @click="updateRolesModal = member; revokeUserModal = null"
                                    v-if="auth.can('editOrganizationMember') && member.id != auth.userProfile.user.id">
                                    {{ $t("dashboard.myOrg.members.updateRoles") }}
                                </BButton>
                            </div>
                        </BTd>
                    </BTr>
                </BTbody>
            </BTableSimple>
        </BCard>
    </BContainer>
</template>

<style lang="scss" scoped>
.users-table {

    td,
    th {
        padding-bottom: .5rem;
    }
}
</style>
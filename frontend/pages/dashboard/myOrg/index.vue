<script setup lang="ts">
import type { ListOrganizationMembersParams } from "bindings/ListOrganizationMembersParams";
import type { ListOrganizationMembersItem } from "bindings/ListOrganizationMembersItem";
import { useOrganizationRepository } from "~/composables/useOrganizationRepository";
import type { ListInvitationsParams } from "bindings/ListInvitationsParams";
const localePath = useLocalePath();
const orgRepo = await useOrganizationRepository();
const auth = await useAuthMandatory();

const canRemoveMember = auth.canComputed('removeOrganizationMember');
const canEditMember = auth.canComputed('editOrganizationMember');

const membersPageNumber = ref<number>(1);
const invitationsPageNumber = ref<number>(1);

const membersFetchParams = ref<ListOrganizationMembersParams>({
    pageNumber: membersPageNumber.value,
    itemsPerPage: 20,
});
const invitationsFetchParams = ref<ListInvitationsParams>({
    pageNumber: invitationsPageNumber.value,
    itemsPerPage: 20,
});

const { data: organizationMembers, refresh: refreshMembers } = await orgRepo.useOrganizationMembers(membersFetchParams);
const { data: invitations, refresh: refreshInvitations } = await orgRepo.usePendingInvitations(invitationsFetchParams);

const revokeUserModal = ref<ListOrganizationMembersItem | null>(null);
const updateRolesModal = ref<ListOrganizationMembersItem | null>(null);
const inviteUserModal = ref<boolean>(false);

const revokeInvitation = async (invitationId: string) => {
    await orgRepo.revokeInvitation(invitationId);
    refreshInvitations();
}
</script>

<template>
    <BContainer>
        <OrganizationRevokeUserModal v-model="revokeUserModal" @ok="refreshMembers" />
        <OrganizationUpdateUserRolesModal v-model="updateRolesModal" @ok="refreshMembers" />
        <OrganizationInviteUserModal v-model="inviteUserModal" @ok="refreshInvitations" />
        <BBreadcrumb>
            <BBreadcrumbItem :to="localePath('/dashboard')">{{ $t("dashboard.mainSidebar.home") }}</BBreadcrumbItem>
            <BBreadcrumbItem active>{{ $t("dashboard.myOrg.pageTitle") }}</BBreadcrumbItem>
        </BBreadcrumb>
        <h2>
            <Icon name="ph:users-four-duotone" />
            {{ $t("dashboard.myOrg.pageTitle") }}
        </h2>
        <p>
            {{ $t('dashboard.myOrg.yourOrganizationIs', {
                organization: auth.userProfile.active_organization.displayName
            }) }}
        </p>
        <BCard class="mb-3">
            <BCardTitle class="mb-3 d-flex justify-content-between">
                {{ $t("dashboard.myOrg.members.title") }}
                <BButton pill variant="outline-primary" size="sm" @click="inviteUserModal = true">
                    {{ $t("dashboard.myOrg.members.inviteUser") }}
                </BButton>
            </BCardTitle>
            <BTableSimple v-if="organizationMembers" class="users-table" responsive>
                <BThead>
                    <BTr>
                        <BTh>{{ $t("dashboard.myOrg.members.name") }}</BTh>
                        <BTh>{{ $t("dashboard.myOrg.members.email") }}</BTh>
                        <BTh>{{ $t("dashboard.myOrg.members.role") }}</BTh>
                        <BTh>{{ $t("dashboard.myOrg.members.actions") }}</BTh>
                    </BTr>
                </BThead>
                <BTbody>
                    <BTr v-for="member in organizationMembers.members" :key="member.id">
                        <BTd>{{ member.firstName }} {{ member.lastName }}
                            <span v-if="member.id == auth.userProfile.user.id">{{ $t("dashboard.myOrg.members.yourself")
                                }}</span>
                        </BTd>
                        <BTd>{{ member.email }}</BTd>
                        <BTd>
                        <div class="d-flex gap-1">
                            <BBadge v-for="role in member.organizationRoles" :key="role">{{ role }}</BBadge>
                        </div>
                        </BTd>
                        <BTd>
                            <div v-if="member.organizationRoles.includes('Owner')" class="text-muted">
                                {{ $t("dashboard.myOrg.members.cannotEditOwner") }}
                            </div>
                            <div class="d-flex gap-2" v-else>
                                <BButton pill variant="outline-danger" size="sm"
                                    @click="revokeUserModal = member; updateRolesModal = null; inviteUserModal = false"
                                    v-if="canRemoveMember && member.id != auth.userProfile.user.id">
                                    {{ $t("dashboard.myOrg.members.revokeUser") }}
                                </BButton>
                                <BButton pill variant="outline-secondary" size="sm"
                                    @click="updateRolesModal = member; revokeUserModal = null; inviteUserModal = false"
                                    v-if="canEditMember && member.id != auth.userProfile.user.id">
                                    {{ $t("dashboard.myOrg.members.updateRoles") }}
                                </BButton>
                            </div>
                        </BTd>
                    </BTr>
                </BTbody>
            </BTableSimple>
        </BCard>

        <BCard class="mb-5" :title="$t('dashboard.myOrg.invitations.title')">
            <BTableSimple v-if="invitations" class="users-table" responsive>
                <BThead>
                    <BTr>
                        <BTh>{{ $t("dashboard.myOrg.invitations.email") }}</BTh>
                        <BTh>{{ $t("dashboard.myOrg.invitations.role") }}</BTh>
                        <BTh>{{ $t("dashboard.myOrg.invitations.actions") }}</BTh>
                    </BTr>
                    <BTr v-for="invitation in invitations" :key="invitation.id">
                        <BTd>{{ invitation.email }}</BTd>
                        <BTd>
                            <BBadge v-for="role in invitation.roles" :key="role">{{ role }}</BBadge>
                        </BTd>
                        <BTd>
                            <BButton pill variant="outline-danger" size="sm" @click="revokeInvitation(invitation.id)"
                                v-if="auth.canComputed('inviteOrganizationMember')">
                                {{ $t("dashboard.myOrg.invitations.revokeInvitation") }}
                            </BButton>
                        </BTd>
                    </BTr>
                </BThead>
            </BTableSimple>
            <div v-if="!invitations?.length" class="text-center text-muted my-5">
                {{ $t("dashboard.myOrg.invitations.noInvitations") }}
            </div>
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
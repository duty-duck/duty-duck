import type { UseFetchOptions } from "#app";
import type { AcceptInvitationCommand } from "bindings/AcceptInvitationCommand";
import type { ChangeMemberRoleCommand } from "bindings/ChangeMemberRoleCommand";
import type { InviteOrganizationMemberCommand } from "bindings/InviteOrganizationMemberCommand";
import type { ListInvitationsParams } from "bindings/ListInvitationsParams";
import type { ListOrganizationMembersParams } from "bindings/ListOrganizationMembersParams";
import type { ListOrganizationMembersResponse } from "bindings/ListOrganizationMembersResponse";
import type { ReceiveInvitationResponse } from "bindings/ReceiveInvitationResponse";
import type { UserInvitation } from "bindings/UserInvitation";

export const useOrganizationRepository = async () => {
    const $fetch = await useServer$fetch();
    const auth = await useAuth();

    return {
        async useOrganizationMembers(params: Ref<ListOrganizationMembersParams> | ListOrganizationMembersParams, options?: UseFetchOptions<ListOrganizationMembersResponse>) {
            return await useServerFetch<ListOrganizationMembersResponse>(`/organizations/${auth.userProfile.active_organization.id}/members`, { params, ...(options ?? {}) })
        },
        async inviteMember(command: InviteOrganizationMemberCommand) {
            return await $fetch(`/organizations/${auth.userProfile.active_organization.id}/members/invite`, { body: command, method: "POST" })
        },
        async removeMember(memberId: string) {
            return await $fetch(`/organizations/${auth.userProfile.active_organization.id}/members/${memberId}`, { method: "DELETE" })
        },
        async changeMemberRole(memberId: string, command: ChangeMemberRoleCommand) {
            return await $fetch(`/organizations/${auth.userProfile.active_organization.id}/members/${memberId}/roles`, { body: command, method: "PUT" })
        },
        async usePendingInvitations(params: Ref<ListInvitationsParams> | ListInvitationsParams, options?: UseFetchOptions<UserInvitation[]>) {
            return await useServerFetch<UserInvitation[]>(`/organizations/${auth.userProfile.active_organization.id}/invitations`, { params, ...(options ?? {}) })
        },
        async revokeInvitation(invitationId: string) {
            return await $fetch(`/organizations/${auth.userProfile.active_organization.id}/invitations/${invitationId}`, { method: "DELETE" })
        },
    }
}

export const usePublicInvitationRepository = async () => {
    const $fetch = await useServer$fetch();

    return {
        async rejectInvitation(organizationId: string, invitationId: string) {
            return await $fetch(`/organizations/${organizationId}/invitations/${invitationId}`, { method: "DELETE" })
        },
        async useInvitation(organizationId: string, invitationId: string) {
            return await useServerFetch<ReceiveInvitationResponse>(`/organizations/${organizationId}/invitations/${invitationId}`)
        },
        async acceptInvitation(organizationId: string, invitationId: string, command: AcceptInvitationCommand) {
            return await $fetch(`/organizations/${organizationId}/invitations/${invitationId}/accept`, { method: "POST", body: command })
        }
    }
}
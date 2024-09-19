import type { ListOrganizationMembersParams } from "bindings/ListOrganizationMembersParams";
import type { ListOrganizationMembersResponse } from "bindings/ListOrganizationMembersResponse";

export const useOrganizationRepository = () => {
    const auth = useAuthMandatory();
    return {
        async useOrganizationMembers(params: Ref<ListOrganizationMembersParams> | ListOrganizationMembersParams) {
            return await useServerFetch<ListOrganizationMembersResponse>(`/organizations/${auth.userProfile.active_organization.id}/members`, { params })
        }
    }
}
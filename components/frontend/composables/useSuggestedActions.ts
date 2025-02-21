export type SuggestedAction = {
    title?: string,
    description?: string[],
    cta?: {
        icon?: string,
        label: string,
        link: string
    }[]
};

export const useSuggestedActions = async () => {
    const { t } = useI18n();
    const localePath = useLocalePath();
    const thisDevice = useThisDevice();
    const auth = await useAuth();

    return computed(() => {
        const actions: SuggestedAction[] = [];

        if (!auth.userProfile.user.phoneNumber || !auth.userProfile.user.phoneNumberVerified) {
            actions.push({
                title: t('dashboard.home.phoneNumberVerificationRequired'),
                description: [t('dashboard.home.phoneNumberVerificationRequiredDescription')],
                cta: [
                    {
                        link: localePath('/dashboard/myAccount'),
                        label: t('dashboard.home.verifyPhoneNumber'),
                        icon: 'ph:phone-call-fill'
                    }
                ]

            })
        }

        if (!thisDevice) {
            actions.push({
                title: t('dashboard.home.pushNotificationsRequired'),
                description: [t('dashboard.home.pushNotificationsRequiredDescription')],
                cta: [
                    {
                        link: localePath('/dashboard/myAccount'),
                        label: t('dashboard.home.configurePushNotifications'),
                        icon: 'ph:bell-duotone'
                    }
                ]

            })
        }

        // todo: allow users to dismiss this action ?
        actions.push({
            title: t('dashboard.home.inviteMembersToOrgActionTitle'),
            description: [t('dashboard.home.inviteMembersToOrgActionDescription')],
            cta: [
                {
                    link: localePath('/dashboard/myOrg'),
                    label: t('dashboard.home.inviteMembersToOrgActionCtaButton'),
                    icon: 'ph:users-duotone'
                }
            ]

        })

        return actions;
    });


}
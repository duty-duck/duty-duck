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
    const monitorRepo = useHttpMonitorRepository();
    const auth = await useAuth();

    const { data: monitorsRes } = await monitorRepo.useHttpMonitors(undefined, { lazy: true, ignoreResponseError: true });

    return computed(() => {
        const actions: SuggestedAction[] = [];

        if (monitorsRes.value?.totalNumberOfResults == 0) {
            actions.push({
                title: t('dashboard.home.createFirstMonitorActionTitle'),
                description: [t('dashboard.home.createFirstMonitorActionDescription')],
                cta: [
                    {
                        link: localePath('/dashboard/httpMonitors/new'),
                        label: t('dashboard.home.createFirstMonitorActionCtaButton'),
                        icon: 'ph:globe-duotone'
                    }
                ]
            })
        }

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
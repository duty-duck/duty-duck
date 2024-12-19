<script setup lang="ts">
const localePath = useLocalePath();
const { tm, t } = useI18n();
const completeFeatureListGroups = computed(() => tm('homepage.completeFeatureList.featureGroups') as { title: string, features: string[] }[]);
const activeSection = ref<string | null>(null);
const sectionsRef = ref<HTMLElement | null>(null);
let observer: IntersectionObserver | null = null;

onMounted(() => {
    if (import.meta.client) {
        observer = new IntersectionObserver((entries) => {
            entries.forEach((entry) => {
                if (entry.isIntersecting) {
                    activeSection.value = entry.target.id;
                    console.log(entry.target.id);
                }
            });
        }, {
            root: null,
            threshold: .6,
            rootMargin: '64px 0px 0px 0px',
        });
        const sections = sectionsRef.value?.children!;
        for (const section of sections) {
            observer.observe(section);
        }
    }
});

onUnmounted(() => {
    if (observer) {
        observer.disconnect();
    }
});
</script>
<template>
    <ShowcaseLayout>
        <section id="hero">
            <ShowcaseHomepageAnimatedBackground />
            <div class="inner">
                <ShowcaseHomepageHeadline />
                <p class="lead fw-bold">{{ $t('homepage.hero.stopGuessing') }}</p>
                <p class="lead text-secondary gray-shadow" v-html="$t('homepage.hero.description')"></p>
                <BButton variant="primary" :to="localePath('/signup')">{{ $t('homepage.hero.cta') }}</BButton>
            </div>
        </section>

        <!-- Sections navigation -->
        <div id="sections-nav" ref="sectionsNavRef">
            <a href="#uptime" class="item" :class="{ active: activeSection === 'uptime' || activeSection === null }">
                <Icon name="ph:globe-duotone" />
                Uptime
            </a>
            <a href="#tasks" class="item" :class="{ active: activeSection === 'tasks' }">
                <Icon name="ph:check-square-offset-duotone" />
                Tâches
            </a>
            <a href="#incidents" class="item" :class="{ active: activeSection === 'incidents' }">
                <Icon name="ph:seal-warning-duotone" />
                Incidents
            </a>
            <a href="#alerts" class="item" :class="{ active: activeSection === 'alerts' }">
                <Icon name="ph:bell-ringing-duotone" />
                Alertes
            </a>
            <a href="#ai" class="item" :class="{ active: activeSection === 'ai' }">
                <Icon name="mdi:stars" />
                AI
            </a>
        </div>

        <!-- Sections content -->
        <div id="sections" ref="sectionsRef">
            <ShowcaseFeatureSection 
                id="uptime" 
                :title="t('homepage.mainSections.uptime.title')"
                :description="t('homepage.mainSections.uptime.description')">
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.uptime.features.checkInterval.title')"
                    :description="t('homepage.mainSections.uptime.features.checkInterval.description')"
                    icon="ph:clock-duotone" 
                    size="small" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.uptime.features.realBrowser.title')"
                    :description="t('homepage.mainSections.uptime.features.realBrowser.description')"
                    icon="ph:google-chrome-logo-duotone" 
                    size="small" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.uptime.features.assertions.title')"
                    :description="t('homepage.mainSections.uptime.features.assertions.description')"
                    icon="ph:check-duotone" 
                    size="large" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.uptime.features.screenshots.title')"
                    :description="t('homepage.mainSections.uptime.features.screenshots.description')"
                    icon="ph:camera-duotone" 
                    size="medium" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.uptime.features.httpDetails.title')"
                    :description="t('homepage.mainSections.uptime.features.httpDetails.description')"
                    icon="ph:code-duotone" 
                    size="small" />
            </ShowcaseFeatureSection>

            <ShowcaseFeatureSection 
                id="tasks" 
                :title="t('homepage.mainSections.tasks.title')"
                :description="t('homepage.mainSections.tasks.description')">
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.tasks.features.cronJobs.title')"
                    :description="t('homepage.mainSections.tasks.features.cronJobs.description')"
                    icon="ph:clock-duotone" 
                    size="large" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.tasks.features.lateNotification.title')"
                    :description="t('homepage.mainSections.tasks.features.lateNotification.description')"
                    icon="ph:person-simple-run-duotone" 
                    size="small" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.tasks.features.integration.title')"
                    :description="t('homepage.mainSections.tasks.features.integration.description')"
                    icon="ph:download-duotone" 
                    size="medium" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.tasks.features.api.title')"
                    :description="t('homepage.mainSections.tasks.features.api.description')"
                    icon="ph:code-duotone" 
                    size="small" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.tasks.features.history.title')"
                    :description="t('homepage.mainSections.tasks.features.history.description')"
                    icon="ph:clock-counter-clockwise-duotone" 
                    size="large" />
            </ShowcaseFeatureSection>

            <ShowcaseFeatureSection 
                id="incidents" 
                :title="t('homepage.mainSections.incidents.title')"
                :description="t('homepage.mainSections.incidents.description')">
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.incidents.features.timeline.title')"
                    :description="t('homepage.mainSections.incidents.features.timeline.description')"
                    icon="ph:clock-duotone" 
                    size="medium" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.incidents.features.collaboration.title')"
                    :description="t('homepage.mainSections.incidents.features.collaboration.description')"
                    icon="ph:users-duotone" 
                    size="small" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.incidents.features.learning.title')"
                    :description="t('homepage.mainSections.incidents.features.learning.description')"
                    icon="ph:lightning-duotone" 
                    size="small" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.incidents.features.organization.title')"
                    :description="t('homepage.mainSections.incidents.features.organization.description')"
                    icon="ph:tag-duotone" 
                    size="small" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.incidents.features.visualization.title')"
                    :description="t('homepage.mainSections.incidents.features.visualization.description')"
                    icon="ph:eye-duotone" 
                    size="medium" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.incidents.features.integration.title')"
                    :description="t('homepage.mainSections.incidents.features.integration.description')"
                    icon="ph:code-duotone" 
                    size="medium" />
            </ShowcaseFeatureSection>

            <ShowcaseFeatureSection 
                id="alerts" 
                :title="t('homepage.mainSections.alerts.title')"
                :description="t('homepage.mainSections.alerts.description')">
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.alerts.features.unlimitedEmails.title')"
                    :description="t('homepage.mainSections.alerts.features.unlimitedEmails.description')"
                    icon="ph:bell-duotone" 
                    size="small" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.alerts.features.sms.title')"
                    :description="t('homepage.mainSections.alerts.features.sms.description')"
                    icon="ph:phone-duotone" 
                    size="small" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.alerts.features.escalation.title')"
                    :description="t('homepage.mainSections.alerts.features.escalation.description')"
                    icon="ph:arrow-up-duotone" 
                    size="large" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.alerts.features.webhooks.title')"
                    :description="t('homepage.mainSections.alerts.features.webhooks.description')"
                    icon="ph:code-duotone" 
                    size="medium" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.alerts.features.weeklyReports.title')"
                    :description="t('homepage.mainSections.alerts.features.weeklyReports.description')"
                    icon="ph:calendar-duotone" 
                    size="small" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.alerts.features.vacation.title')"
                    :description="t('homepage.mainSections.alerts.features.vacation.description')"
                    icon="ph:island-duotone" 
                    size="medium" />
            </ShowcaseFeatureSection>

            <ShowcaseFeatureSection 
                id="ai" 
                :title="t('homepage.mainSections.ai.title')"
                :description="t('homepage.mainSections.ai.description')">
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.ai.features.detection.title')"
                    :description="t('homepage.mainSections.ai.features.detection.description')"
                    icon="ph:eyeglasses-duotone" 
                    size="small" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.ai.features.fixes.title')"
                    :description="t('homepage.mainSections.ai.features.fixes.description')"
                    icon="ph:lightning-duotone" 
                    size="small" />
                <ShowcaseFeatureBlock 
                    :title="t('homepage.mainSections.ai.features.postMortems.title')"
                    :description="t('homepage.mainSections.ai.features.postMortems.description')"
                    icon="ph:clipboard-text-duotone" 
                    size="large" />
            </ShowcaseFeatureSection>

            <!-- Final section (removes the primary color highlight on the sections navigation when intersecting)-->
            <section class="container" id="final-section">
                <div class="mt-3 mb-5 text-center">
                    <h2>{{ $t('homepage.final.title') }}</h2>
                    <p class="lead">{{ $t('homepage.final.description') }}</p>
                    <BButton variant="primary" :to="localePath('/signup')">{{ $t('homepage.final.cta') }}</BButton>
                </div>
                <div class="my-5 text-center">
                    <h2 class="text-center fs-5">{{ $t('homepage.completeFeatureList.title') }}</h2>
                </div>
                <div class="row mb-5">
                    <div class="col-sm-2 col-md-4 col-xl-3" v-for="group in completeFeatureListGroups" :key="group.title">
                        <ul class="list-unstyled">
                            <li class="fw-bold mb-2 text-secondary">{{ group.title }}</li>
                            <ul>
                                <li v-for="feature in group.features" :key="feature">{{ feature }}</li>
                            </ul>
                        </ul>
                    </div>
                </div>
            </section>
        </div>
    </ShowcaseLayout>
</template>

<style scoped lang="scss">
@import "~/assets/main.scss";
$sections-nav-height: 70px;

#hero {
    @extend .px-2, .py-4;
    position: relative;
    min-height: 65vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: left;

    .inner {
        position: relative;
        z-index: 10;
    }

    @include media-breakpoint-up(md) {
        text-align: center;

        p.lead {
            font-size: 1.5rem;
        }
    }

    p.lead {
        @extend .mt-3;
        font-size: 1.2rem;
    }

    background-color: #f0f0f0;
}

.gray-shadow {
    text-shadow: 0 0 15px rgba(134, 134, 134, 0.25);
}

#sections-nav {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
    height: $sections-nav-height;
    position: sticky;
    top: 0;
    background-color: $white;
    border-bottom: 1px solid rgb(234 236 241);
    border-top: 1px solid rgb(234 236 241);
    overflow-x: auto;
    z-index: 100;

    @include media-breakpoint-up(md) {
        gap: 2rem;
    }

    .item {
        .iconify {
            font-size: 1.2rem;
        }

        padding: .75rem 0;
        text-decoration: none;
        color: $gray-600;
        gap: .1rem;
        display: flex;
        flex-direction: column;
        align-items: center;

        &.active {
            color: $primary;
        }
    }
}
</style>
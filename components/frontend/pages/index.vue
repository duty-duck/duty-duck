<script setup lang="ts">
const localePath = useLocalePath();
const { tm } = useI18n();
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
            <ShowcaseFeatureSection id="uptime" title="Uptime monitoring that just works"
                description="Be the first to know when you website is down">
                <ShowcaseFeatureBlock title="15 seconds check interval"
                    description="Frequent checks mean we can detect issues in real-time and help you fix them quickly"
                    icon="ph:clock-duotone" size="small" />
                <ShowcaseFeatureBlock title="Real browser"
                    description="We use a real browser to check your website, with Javascript rendering and all"
                    icon="ph:google-chrome-logo-duotone" size="small" />
                <ShowcaseFeatureBlock title="Flexible content assertions"
                    description="You can define custom assertions to ensure your endpoints not only respond, but also serve the expected content"
                    icon="ph:check-duotone" size="large" />
                <ShowcaseFeatureBlock title="Screenshots"
                    description="We take screenshots of your website on failure, so you can see exactly what your users experienced"
                    icon="ph:camera-duotone" size="medium" />
                <ShowcaseFeatureBlock title="Complete HTTP Response details"
                    description="We capture the entire HTTP response, including headers, status code, and body"
                    icon="ph:code-duotone" size="small" />
            </ShowcaseFeatureSection>
            <ShowcaseFeatureSection id="tasks" title="Task monitoring"
                description="Monitor the status of any process, one-shot or recurring, and get notified when a task is failing, running late, or just not running at all. No more missed database backups.">
                <ShowcaseFeatureBlock title="Keep track of any Cron job"
                    description="Workflows, database backups, daily reports, etc." icon="ph:clock-duotone"
                    size="large" />
                <ShowcaseFeatureBlock title="Get notified when a task is running late"
                    description="A job isn't starting on time? We'll let you know" icon="ph:person-simple-run-duotone"
                    size="small" />
                <ShowcaseFeatureBlock title="Easy integration"
                    description="Our agent can monitor any process. Just give it the command to run, and we'll take care of the rest"
                    icon="ph:download-duotone" size="medium" />
                <ShowcaseFeatureBlock title="Simple API for developers"
                    description="Need a custom integration? Our REST API can do anything you can do from the dashboard"
                    icon="ph:code-duotone" size="small" />
                <ShowcaseFeatureBlock title="Complete job history"
                    description="You can see the previous runs of any job, and for each job, get the output, logs and timings and ease"
                    icon="ph:clock-counter-clockwise-duotone" size="large" />
            </ShowcaseFeatureSection>
            <ShowcaseFeatureSection id="incidents" title="Incidents"
                description="Collect insights on past and ongoing incidents, understand what went wrong and know who is taking care of it.">
                <ShowcaseFeatureBlock title="Get the complete incident timeline"
                    description="Know when an incident was detected, when it was confirmed, who worked on it, and when it was resolved"
                    icon="ph:clock-duotone" size="medium" />
                <ShowcaseFeatureBlock title="Collaborate on incidents"
                    description="Work together on incidents, with comments, status updates, and assignees"
                    icon="ph:users-duotone" size="small" />
                <ShowcaseFeatureBlock title="Learn from past incidents"
                    description="When an incident occurs, we can find similar incidents in the past and suggest fixes from what worked before"
                    icon="ph:lightning-duotone" size="small" />
                <ShowcaseFeatureBlock title="Organize the chaos"
                    description="With incident grouping and labels, you can organize incidents by cause, application, region, team, or anything you see fit"
                    icon="ph:tag-duotone" size="small" />
                <ShowcaseFeatureBlock title="Visualize your uptime at a glance"
                    description="With public and private status pages, get a quick overview of your incident history, grouped by any criteria you want"
                    icon="ph:eye-duotone" size="medium" />
                <ShowcaseFeatureBlock title="Integrate incidents from other systems"
                    description="With manual incident creation from our dashboard, or with our easy-to-use API, put all your incident data in one place"
                    icon="ph:code-duotone" size="medium" />
            </ShowcaseFeatureSection>


            <ShowcaseFeatureSection id="alerts" title="Alerts"
                description="When things go wrong, we make sure the right people are alerted at exactly the right time. Working in teams? Fine-tune your alerting to your team's workflow.">
                <ShowcaseFeatureBlock title="Unlimited e-mails and push notifications"
                    description="Alerts right in your pocket, no limit, no extra cost." icon="ph:bell-duotone"
                    size="small" />
                <ShowcaseFeatureBlock title="SMS alerts and phone calls"
                    description="Know what's going on, even when offline" icon="ph:phone-duotone" size="small" />
                <ShowcaseFeatureBlock title="Flexible escalation policies"
                    description="Make sure incidents are taken care of: define first responders, escalate stagnating incidents, mix alerting channels, and more."
                    icon="ph:arrow-up-duotone" size="large" />
                <ShowcaseFeatureBlock title="Push alerts to any system"
                    description="With easy-to-setup webhooks, receive alerts right in a messaging app, a terminal, on a billboard, or anything else you can think of"
                    icon="ph:code-duotone" size="medium" />
                <ShowcaseFeatureBlock title="Receive weekly reports"
                    description="Get a weekly report of all your incidents, with a summary of what happened, and what was fixed"
                    icon="ph:calendar-duotone" size="small" />
                <ShowcaseFeatureBlock title="Someone going on vacation?"
                    description="Let your team members turn off their notifications, and escalate automatically to the next person in charge"
                    icon="ph:island-duotone" size="medium" />
            </ShowcaseFeatureSection>
            <ShowcaseFeatureSection id="ai" title="AI"
                description="We use AI to help you understand your incidents, and to help you fix them.">
                <ShowcaseFeatureBlock title="Similar incidents detection"
                    description="We can organize similar incidents together" icon="ph:eyeglasses-duotone"
                    size="small" />
                <ShowcaseFeatureBlock title="Suggested fixes"
                    description="We can suggest fixes for an incident, based on what worked before for similar incidents"
                    icon="ph:lightning-duotone" size="small" />
                <ShowcaseFeatureBlock title="Automatically-generated post-mortems"
                    description="We can summarize an incident, and provide a quick overview of what happened, and what was fixed. We can send you that post-mortem by e-mail and let you use natural language to search in previous post-mortems."
                    icon="ph:clipboard-text-duotone" size="large" />
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
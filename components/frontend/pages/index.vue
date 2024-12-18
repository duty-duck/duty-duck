<script setup lang="ts">
const localePath = useLocalePath();
const activeSection = ref<string | null>(null);
const sectionsRef = ref<HTMLElement | null>(null);
let observer: IntersectionObserver | null = null;

onMounted(() => {
    if (import.meta.client) {
        observer = new IntersectionObserver((entries) => {
            entries.forEach((entry) => {
                if (entry.isIntersecting) {
                    activeSection.value = entry.target.id;
                }
            });
        }, {
            root: null,
            threshold: .8,
            rootMargin: '64px 0px 0px 0px'
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

const uptimeFeatures = [
    {
        title: 'Real-time Monitoring',
        description: 'Monitor your applications with precision and ease. Get instant alerts and detailed insights when issues arise.',
        icon: 'ph:globe-duotone',
        iconBgColor: 'bg-success bg-opacity-10',
        size: 'large'
    },
    {
        title: 'Real-time Monitoring',
        description: 'Monitor your applications with precision and ease. Get instant alerts and detailed insights when issues arise.',
        icon: 'ph:globe-duotone',
        iconBgColor: 'bg-success bg-opacity-10',
        size: 'large'
    },
    {
        title: "Performance Metrics",
        description: "Track response times, error rates, and system resources in real-time.",
        icon: 'ph:activity-duotone',
        size: "large",
        iconBgColor: "bg-primary bg-opacity-10",
        iconColor: "text-primary"
  },
  {
        title: "24/7 Monitoring",
        description: "Round-the-clock monitoring ensures you never miss a critical event.",
        icon: 'ph:clock-duotone',
        size: "small",
        iconBgColor: "bg-info bg-opacity-10",
        iconColor: "text-info"
  },
  {
    title: "Customizable Alerts",
    description: "Create tailored alerts for specific events or conditions.",
    icon: 'ph:bell-duotone',
    size: "small",
    iconBgColor: "bg-warning bg-opacity-10",
    iconColor: "text-warning"
  }
];
</script>
<template>
    <ShowcaseLayout>
        <section id="hero">
            <ShowcaseHomepageAnimatedBackground />
            <div class="inner">
                <ShowcaseHomepageHeadline />
                <p class="lead fw-bold">{{ $t('homepage.hero.stopGuessing') }}</p>
                <p class="lead text-secondary gray-shadow">
                    Que vous soyez un dev solo ou une équipe qui cartonne,<br />
                    on surveille votre site et toutes vos tâches d'arrière plan,<br>
                    on alerte la bonne personne, au bon moment,
                    <span class="text-body-emphasis d-block mt-2">bref, on vous simplifie la vie</span>
                </p>
                <BButton variant="primary" :to="localePath('/signup')">Commencer gratuitement</BButton>
            </div>

        </section>

        <!-- Sections navigation -->
        <div id="sections-nav" ref="sectionsNavRef">
            <a href="#uptime" class="item" :class="{ active: activeSection === 'uptime' || !activeSection }">
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
                title="Real-time Monitoring"
                description="Monitor your applications with precision and ease. Get instant alerts and detailed insights when issues arise."
                :features="uptimeFeatures"
            />
            <ShowcaseFeatureSection
                id="tasks"
                title="Task monitoring"
                description="Monitor your applications with precision and ease. Get instant alerts and detailed insights when issues arise."
                :features="uptimeFeatures"
            />


            <section id="incidents">
                <h2>Incidents</h2>
                <p>
                    Lorem ipsum dolor sit amet consectetur adipisicing elit. Quisquam, quos.
                </p>
            </section>
            <section id="alerts">
                <h2>Alertes</h2>
                <p>
                    Lorem ipsum dolor sit amet consectetur adipisicing elit. Quisquam, quos.
                </p>
            </section>
            <section id="ai">
                <h2>AI</h2>
                <p>
                    Lorem ipsum dolor sit amet consectetur adipisicing elit. Quisquam, quos.
                </p>
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
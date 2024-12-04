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
</script>
<template>
    <ShowcaseLayout>
        <section id="hero">
            <h1>On garde un œil sur vos systèmes</h1>
            <p class="lead">
                Que vous soyez un dev solo ou une équipe qui cartonne,<br />notre plateforme d'observabilité garde
                vos
                systèmes au top, sans vider votre portefeuille.
                <br />On vous simplifie la vie en quelques clics !
            </p>
            <p>14 jours d'essai gratuits, ça vous tente ?</p>
            <BButton variant="primary" :to="localePath('/signup')">Commencer</BButton>
        </section>

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
        <div id="sections" ref="sectionsRef">
            <section id="uptime">
                <h2>Uptime</h2>
                <p>
                    Lorem ipsum dolor sit amet consectetur adipisicing elit. Quisquam, quos.
                </p>
            </section>
            <section id="tasks">
                <h2>Tâches</h2>
                <p>
                    Lorem ipsum dolor sit amet consectetur adipisicing elit. Quisquam, quos.
                </p>
            </section>
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
    min-height: 65vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: left;
    min-height: calc(100vh - $sections-nav-height - $navbar-height - $settings-bar-height);

    @include media-breakpoint-up(md) {
        text-align: center;

        h1 {
            font-size: 3.5rem;
            line-height: 5rem;
        }

        p.lead {
            font-size: 1.5rem;
        }
    }

    h1 {
        @include homepage-heading;
        font-size: 2.5rem;
        background: linear-gradient(to right, $info, $primary);
        background-clip: text;
        color: transparent;
    }

    p.lead {
        @extend .my-4;
        font-size: 1.2rem;
    }

    background-color: #f0f0f0;
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

#sections {
    section {
        padding-top: 4.5rem;
        min-height: 80vh;
        border-bottom: 1px solid $gray-200;
    }
}
</style>

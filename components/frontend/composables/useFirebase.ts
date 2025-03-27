import { initializeApp, type FirebaseApp } from "firebase/app";
import { getMessaging, getToken as getFirebaseToken, onMessage as firebaseOnMessage, type MessagePayload, isSupported } from "firebase/messaging";
import { createSharedComposable } from "@vueuse/core";
import { useToastController } from "bootstrap-vue-next";

// TODO: Add SDKs for Firebase products that you want to use
// https://firebase.google.com/docs/web/setup#available-libraries

// Initialize Firebase
let app: FirebaseApp | undefined;

export const useFirebaseMessageHandler = () => {
    const { show } = useToastController();
    return (payload: MessagePayload) => {
        console.log("Received a new Firebase message:", payload);
        show?.({
            props: {
                body: payload.notification?.body,
                title: payload.notification?.title,
                variant: "light",
                value: 10000,
            },

        });
    }
}

const useApp = (): FirebaseApp => {
    if (app) {
        return app;
    }
    const config = useRuntimeConfig();
    app = initializeApp({
        apiKey: config.public.firebaseApiKey,
        authDomain: config.public.firebaseAuthDomain,
        projectId: config.public.firebaseProjectId,
        storageBucket: config.public.firebaseStorageBucket,
        messagingSenderId: config.public.firebaseMessagingSenderId,
        appId: config.public.firebaseAppId,
    });
    return app;
}

export type TokenState = null | "loading" | { token: string };

export const useFirebaseMessaging = createSharedComposable(async () => {
    if (!(await isSupported())) {
        console.warn("Firebase messaging is not supported in this browser");
        return null;
    }

    let serviceWorkerRegistration = null as ServiceWorkerRegistration | null;
    const messaging = getMessaging(useApp());
    const thisDeviceToken = ref<TokenState>(null);

    const registerMessageHandler = (handler: (payload: MessagePayload) => void) => {
        firebaseOnMessage(messaging, handler);
        console.log("Registered a new Firebase message handler")
    }

    /** 
     * Register service worker if needed, and returns the registration
     */
    const registerServiceWorker = async () => {
        if (serviceWorkerRegistration) return serviceWorkerRegistration;

        const serviceWorkerURL = new URL("/firebase-messaging-sw", document.baseURI).href
        console.log("Registering service worker at:", serviceWorkerURL);
        serviceWorkerRegistration = await navigator.serviceWorker.register(serviceWorkerURL, { type: 'classic' });
        return serviceWorkerRegistration;
    }

    const fetchThisDeviceToken = async () => {
        const { public: { firebaseVapidKey } } = useRuntimeConfig();
        thisDeviceToken.value = "loading";
        try {
            const serviceWorkerRegistration = await registerServiceWorker();
            const res = await getFirebaseToken(messaging, { vapidKey: firebaseVapidKey, serviceWorkerRegistration }) || null;
            if (res) {
                console.log("Push notification token:", res);
                thisDeviceToken.value = { token: res }
            } else {
                thisDeviceToken.value = null
            }
        } catch (e) {
            thisDeviceToken.value = null;
        }
    }

    /**
     * Asks user for permission to send notfications and stores the token in the store 
     * @returns a promise indicating whether the permission was successfully granted
     */
    const requestPermission = async (): Promise<boolean> => {
        thisDeviceToken.value = "loading";
        console.log('Requesting permission...');
        const permission = await Notification.requestPermission();
        if (permission === 'granted') {
            console.log('Notification permission granted.');
            await fetchThisDeviceToken();
            return true
        } else {
            console.log('Unable to get permission to notify.');
            await fetchThisDeviceToken();
            return false;
        }
    }

    // When the store is first initialized, load the token in the background
    fetchThisDeviceToken();

    return reactive({
        token: thisDeviceToken,
        requestPermission,
        registerMessageHandler
    })
})
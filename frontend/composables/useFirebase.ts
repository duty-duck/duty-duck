import { initializeApp, type FirebaseApp, type FirebaseOptions } from "firebase/app";
import { getMessaging, getToken as getFirebaseToken, onMessage as firebaseOnMessage, type MessagePayload } from "firebase/messaging";
import serviceWorkerUrl from "@/assets/firebase-messaging-sw.js?worker&url";

// TODO: Add SDKs for Firebase products that you want to use
// https://firebase.google.com/docs/web/setup#available-libraries

// Initialize Firebase
let app: FirebaseApp | undefined;

export const useFirebaseMessageHandler = () => {
    const { show } = useToast();
    return (payload: MessagePayload) => {
        show?.({
            props: {
                body: payload.notification?.body,
                title: payload.notification?.title,
                variant: "secondary",
                value: 60000
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

export const useFirebaseMessaging = defineStore('firebaseMessaging', () => {
    const messaging = getMessaging(useApp());
    const token = ref<TokenState>(null);

    const onMessage = (handler: (payload: MessagePayload) => void) => {
        console.log("Registered a new Firebase message handler")
        firebaseOnMessage(messaging, handler);
    }

    const getToken = async () => {
        const { public: { firebaseVapidKey } } = useRuntimeConfig();
        token.value = "loading";
        try {
            const absServiceWorkerUrl = new URL(serviceWorkerUrl, document.baseURI).href
            const serviceWorkerUrlWithConfig = new URL(`/api/injectConfig?url=${encodeURIComponent(absServiceWorkerUrl)}`, document.baseURI).href;
            console.log("Registering service worker at:", serviceWorkerUrlWithConfig);
            const serviceWorkerRegistration = await navigator.serviceWorker.register(serviceWorkerUrlWithConfig, { type: 'module' });
            const res = await getFirebaseToken(messaging, { vapidKey: firebaseVapidKey, serviceWorkerRegistration }) || null;
            if (res) {
                console.log("Push notification token:", res);
                token.value = { token: res }
            } else {
                token.value = null
            }
        } catch (e) {
            token.value = null;
        }
    }

    /**
     * Asks user for permission to send notfications and stores the token in the store 
     * @returns a promise indicating whether the permission was successfully granted
     */
    const requestPermission = async (): Promise<boolean> => {
        token.value = "loading";
        console.log('Requesting permission...');
        const permission = await Notification.requestPermission();
        if (permission === 'granted') {
            console.log('Notification permission granted.');
            await getToken();
            return true
        } else {
            console.log('Unable to get permission to notify.');
            await getToken();
            return false;
        }
    }

    // When the store is first initialized, load the token in the background
    getToken();

    return {
        token,
        requestPermission,
        onMessage
    }
});
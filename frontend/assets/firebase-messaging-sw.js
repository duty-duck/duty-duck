import { initializeApp } from "firebase/app";
import { getMessaging, onBackgroundMessage } from "firebase/messaging/sw";

// config will be injected at runtime when downloading the service worker
const config = {};

console.log("SW Config:", config);

// Initialize the Firebase app in the service worker by passing in
// your app's Firebase config object.
// https://firebase.google.com/docs/web/setup#config-object
const firebaseApp = initializeApp(config.firebase);

// Retrieve an instance of Firebase Messaging so that it can handle background
// messages.
const messaging = getMessaging(firebaseApp);

onBackgroundMessage(messaging, (payload) => {
  console.log(
    "[firebase-messaging-sw.js] Received background message ",
    payload
  );
  // Customize notification here
  const notificationOptions = {
    body: payload.notification.body,
    icon: "/navbar-duck.png",
  };

  self.registration.showNotification(
    payload.notification.title,
    notificationOptions
  );
});

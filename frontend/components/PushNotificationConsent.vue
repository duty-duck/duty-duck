<script lang="ts" setup>
import type { RegisterUserDeviceCommand } from "bindings/RegisterUserDeviceCommand";
import type { UserDevice } from "bindings/UserDevice";

const { show } = useToast();
const firebaseMessaging = useFirebaseMessaging();
const devicesRepository = useUserDevicesRepository();
const thisDevice = await useThisDevice();
const thisDeviceType = useThisDeviceType();
const newDeviceName = ref("");
const {
  data: devicesRes,
  refresh: refreshDevices,
  status: devicesStatus,
} = await devicesRepository.useDevices();

const deviceIcon = (device: UserDevice) =>
  device.deviceType == "desktop" ? "ph:desktop-fill" : "ph:device-mobile-fill";

const enableNotifications = async () => {
  show?.({
    props: {
      body: "Asking for permission",
    },
  });

  const res = await firebaseMessaging.requestPermission();
  if (res && firebaseMessaging.token && firebaseMessaging.token !== "loading") {
    const { token } = firebaseMessaging.token;

    const command: RegisterUserDeviceCommand = {
      label: newDeviceName.value,
      pushNotificationToken: token,
      deviceType: thisDeviceType,
    };

    await devicesRepository.registerDevice(command);
    await refreshDevices();

    show?.({
      props: {
        body: "Success",
      },
    });
  } else {
    show?.({
      props: {
        title: "Failed",
        body: "Failed",
      },
    });
  }
};

const removeDevice = async (deviceId: string) => {
  await devicesRepository.removeDevice(deviceId);
  await refreshDevices();
};
</script>
<template>
  <BCard no-body>
    <BCardBody>
      <BCardTitle>{{ $t("dashboard.myAccount.pushNotifications") }}</BCardTitle>
      <div
        v-if="
          firebaseMessaging.token == 'loading' || devicesStatus == 'pending'
        "
      >
        <BPlaceholder size="sm" animation="glow" cols="10" />
        <BPlaceholder size="sm" animation="glow" cols="8" />
      </div>
      <div v-else-if="thisDevice">
        Vous avez déjà accepté les notifications Push
      </div>
      <div v-else>
        <p>
          Les notifications push vous permettent d'être informé en temps réel,
          sur votre appareil, des incidents.
        </p>
        <div class="row">
          <div class="col-lg-7 mb-2">
            <BFormInput
              id="deviceName"
              placeholder="Nom de cet appareil"
              v-model="newDeviceName"
            />
          </div>
          <div class="col-lg-5">
            <BButton
              variant="primary"
              @click="enableNotifications"
              :disabled="!newDeviceName"
            >
              <Icon name="ph:bell" />
              Activer les notifications push
            </BButton>
          </div>
        </div>
      </div>
    </BCardBody>
    <BListGroup flush v-if="devicesRes?.devices.length">
      <BListGroupItem
        v-for="device in devicesRes?.devices"
        :key="device.id"
        class="d-flex justify-content-between align-items-center"
      >
        <div>
          <Icon :name="deviceIcon(device)" />
          {{ device.label }}
          <span v-if="device.id == thisDevice?.id"> (this device)</span>
        </div>
        <div>
          <BButton size="sm" @click="removeDevice(device.id)">
            Remove device
          </BButton>
        </div>
      </BListGroupItem>
    </BListGroup>
  </BCard>
</template>

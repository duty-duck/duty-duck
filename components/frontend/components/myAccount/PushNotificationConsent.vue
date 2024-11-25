<script lang="ts" setup>
import type { RegisterUserDeviceCommand } from "bindings/RegisterUserDeviceCommand";
import type { UserDevice } from "bindings/UserDevice";

const { show } = useToast();
const { t } = useI18n();
const firebaseMessaging = useFirebaseMessaging();
const devicesRepository = await useUserDevicesRepository();
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
      body: t("dashboard.pushNotifications.askingPermission"),
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
        body: t("dashboard.pushNotifications.success"),
      },
    });
  } else {
    show?.({
      props: {
        title: t("dashboard.pushNotifications.failed"),
        body: t("dashboard.pushNotifications.failed"),
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
      <BCardTitle>{{ $t("dashboard.pushNotifications.title") }}</BCardTitle>
      <div
        v-if="
          firebaseMessaging.token == 'loading' || devicesStatus == 'pending'
        "
      >
        {{ $t("dashboard.pushNotifications.loading") }}
      </div>
      <div v-else-if="thisDevice">
        {{ $t("dashboard.pushNotifications.alreadyAccepted") }}
      </div>
      <div v-else>
        <p>
          {{ $t("dashboard.pushNotifications.description") }}
        </p>
        <div class="row">
          <div class="col-lg-7 mb-2">
            <BFormInput
              id="deviceName"
              :placeholder="$t('dashboard.pushNotifications.deviceNamePlaceholder')"
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
              {{ $t("dashboard.pushNotifications.enableButton") }}
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
          <span v-if="device.id == thisDevice?.id">{{ $t("dashboard.pushNotifications.thisDevice") }}</span>
        </div>
        <div>
          <BButton size="sm" @click="removeDevice(device.id)">
            {{ $t("dashboard.pushNotifications.removeDevice") }}
          </BButton>
        </div>
      </BListGroupItem>
    </BListGroup>
  </BCard>
</template>

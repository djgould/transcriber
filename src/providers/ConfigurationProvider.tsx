import {
  useAudioInputDevicesQuery,
  useAudioOutputDevicesQuery,
} from "@/hooks/useMediaDevices";
import { useEffect } from "react";
import { useAtom } from "jotai";
import {
  selectedAudioInputDeviceAtom,
  selectedAudioOutputDeviceAtom,
} from "@/atoms/audioDeviceAtom";

export function ConfigurationProvider({ children }: React.PropsWithChildren) {
  const audioInputDevices = useAudioInputDevicesQuery();
  const audioOutputDevices = useAudioOutputDevicesQuery();

  const [selectedAudioInputDevice, setSelectedAudioInputDevice] = useAtom(
    selectedAudioInputDeviceAtom
  );
  const [selectedAudioOutputDevice, setSelectedAudioOutputDevice] = useAtom(
    selectedAudioOutputDeviceAtom
  );

  useEffect(() => {
    if (!selectedAudioInputDevice && audioInputDevices.data)
      setSelectedAudioInputDevice(audioInputDevices.data[0]);
  }, audioInputDevices.data);

  useEffect(() => {
    if (!selectedAudioOutputDevice && audioOutputDevices.data)
      setSelectedAudioOutputDevice(audioOutputDevices.data[0]);
  }, audioOutputDevices.data);
  return <>{children}</>;
}

import { useAudioDevicesQuery } from "@/hooks/useMediaDevices";
import { useEffect } from "react";
import { useAtom } from "jotai";
import { selectedAudioDeviceAtom } from "@/atoms/audioDeviceAtom";

export function ConfigurationProvider({ children }: React.PropsWithChildren) {
  const audioDevices = useAudioDevicesQuery();
  const [selectedAudioDevice, setSelectedAudioDevice] = useAtom(
    selectedAudioDeviceAtom
  );
  useEffect(() => {
    if (!selectedAudioDevice && audioDevices.data)
      setSelectedAudioDevice(audioDevices.data[0]);
  }, audioDevices.data);
  return <>{children}</>;
}
